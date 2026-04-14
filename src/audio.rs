use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::sync::{Condvar, Mutex};
#[cfg(windows)]
use windows::core::Interface;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
#[cfg(windows)]
use windows::Win32::Media::Audio::*;
#[cfg(windows)]
use windows::Win32::System::Com::*;
#[cfg(windows)]
use windows::Win32::System::Threading::{
    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_ABOVE_NORMAL, *,
};

#[derive(Debug, Clone)]
pub enum StreamState {
    Disconnected,
    Connecting,
    Connected,
    Streaming,
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub bytes_sent: u64,
    pub bitrate_kbps: f64,
    pub uptime: Duration,
    pub client_latency_ms: f64,
    pub drops: u32,
    pub capture_format: String,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    StateChanged(StreamState),
    Log(String),
    StatsUpdated(Stats),
    VolumeChanged { volume: f32, muted: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureMode {
    WasapiLoopback,
    VbCable,
}

pub struct StreamConfig {
    pub server: String,
    pub port: u16,
    pub rate: u32,
    pub channels: u16,
    pub device_id: Option<String>,
    pub process_id: Option<u32>,
    pub mute_local_output: bool,
    pub capture_mode: CaptureMode,
}

pub struct AudioStreamer {
    event_tx: flume::Sender<AudioEvent>,
    event_rx: flume::Receiver<AudioEvent>,
    running: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AudioStreamer {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();
        Self {
            event_tx: tx,
            event_rx: rx,
            running: Arc::new(AtomicBool::new(false)),
            thread: None,
        }
    }

    pub fn event_receiver(&self) -> flume::Receiver<AudioEvent> {
        self.event_rx.clone()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn start(&mut self, cfg: StreamConfig) {
        if self.running.load(Ordering::Acquire) {
            return;
        }
        self.running.store(true, Ordering::Release);

        let tx = self.event_tx.clone();
        let running = self.running.clone();

        self.thread = Some(
            std::thread::Builder::new()
                .name("pulse-stream-audio".to_string())
                .spawn(move || {
                    run_loop(&tx, &running, &cfg);
                })
                .expect("failed to spawn audio thread"),
        );
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Default for AudioStreamer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioStreamer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
pub fn get_output_devices() -> Vec<DeviceInfo> {
    let mut devices = vec![DeviceInfo {
        id: String::new(),
        name: "Default".to_string(),
    }];

    unsafe {
        // winit hasn't initialized COM yet when Application::new runs.
        // Initialize as STA (compatible with winit's later OleInitialize).
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let co_result =
            CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL);

        let Ok(enumerator) = co_result else {
            return devices;
        };

        let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) else {
            return devices;
        };

        let Ok(count) = collection.GetCount() else {
            return devices;
        };

        for i in 0..count {
            if let Ok(device) = collection.Item(i) {
                let id_str = device
                    .GetId()
                    .ok()
                    .map(|id| {
                        let s = id.to_string().unwrap_or_default();
                        CoTaskMemFree(Some(id.0 as *const _));
                        s
                    })
                    .unwrap_or_default();

                let name =
                    get_device_name(&device).unwrap_or_else(|| format!("Audio Device {}", i + 1));

                devices.push(DeviceInfo { id: id_str, name });
            }
        }
    }

    devices
}

#[cfg(not(windows))]
pub fn get_output_devices() -> Vec<DeviceInfo> {
    vec![DeviceInfo {
        id: String::new(),
        name: "Default".to_string(),
    }]
}

#[cfg(windows)]
pub fn detect_vb_cable() -> Option<DeviceInfo> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let enumerator =
            CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .ok()?;

        let collection = enumerator
            .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
            .ok()?;

        let count = collection.GetCount().ok()?;

        for i in 0..count {
            let Ok(device) = collection.Item(i) else {
                continue;
            };
            let name = get_device_name(&device)?;
            if name.to_uppercase().contains("CABLE") {
                let id_str = device
                    .GetId()
                    .ok()
                    .map(|id| {
                        let s = id.to_string().unwrap_or_default();
                        CoTaskMemFree(Some(id.0 as *const _));
                        s
                    })
                    .unwrap_or_default();
                return Some(DeviceInfo { id: id_str, name });
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub fn detect_vb_cable() -> Option<DeviceInfo> {
    None
}

/// Find the VB-CABLE **render** device (the playback side that apps send audio to).
#[cfg(windows)]
pub fn detect_vb_cable_render() -> Option<DeviceInfo> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let enumerator =
            CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .ok()?;

        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .ok()?;

        let count = collection.GetCount().ok()?;

        for i in 0..count {
            let Ok(device) = collection.Item(i) else {
                continue;
            };
            let name = get_device_name(&device)?;
            if name.to_uppercase().contains("CABLE") {
                let id_str = device
                    .GetId()
                    .ok()
                    .map(|id| {
                        let s = id.to_string().unwrap_or_default();
                        CoTaskMemFree(Some(id.0 as *const _));
                        s
                    })
                    .unwrap_or_default();
                return Some(DeviceInfo { id: id_str, name });
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub fn detect_vb_cable_render() -> Option<DeviceInfo> {
    None
}

/// Get the current default audio endpoint ID for a given role.
#[cfg(windows)]
unsafe fn get_default_endpoint_id(enumerator: &IMMDeviceEnumerator, role: ERole) -> Option<String> {
    let device = enumerator.GetDefaultAudioEndpoint(eRender, role).ok()?;
    let pwstr = device.GetId().ok()?;
    let s = pwstr.to_string().unwrap_or_default();
    CoTaskMemFree(Some(pwstr.0 as *const _));
    Some(s)
}

/// Set the default audio endpoint for all roles using the undocumented IPolicyConfig interface.
/// This is the same approach used by every audio-switching tool on Windows (stable since Vista).
#[cfg(windows)]
unsafe fn set_default_endpoint(device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let clsid = windows::core::GUID::from_values(
        0x870AF99C,
        0x171D,
        0x4F9E,
        [0xAF, 0x0D, 0xE6, 0x3D, 0xF4, 0x0C, 0x2B, 0xC9],
    );
    let iid = windows::core::GUID::from_values(
        0xF8679F50,
        0x850A,
        0x41CF,
        [0x9C, 0x72, 0x43, 0x0F, 0x29, 0x02, 0x90, 0xC8],
    );

    extern "system" {
        fn CoCreateInstance(
            rclsid: *const windows::core::GUID,
            punkouter: *const std::ffi::c_void,
            dwclscontext: u32,
            riid: *const windows::core::GUID,
            ppv: *mut *mut std::ffi::c_void,
        ) -> windows::core::HRESULT;
    }

    let mut obj: *mut std::ffi::c_void = std::ptr::null_mut();
    CoCreateInstance(&clsid, std::ptr::null(), CLSCTX_ALL.0, &iid, &mut obj).ok()?;

    if obj.is_null() {
        return Err("IPolicyConfig creation failed".into());
    }

    let vtable = *(obj as *const *const usize);

    type SetDefaultEndpointFn =
        unsafe extern "system" fn(*mut std::ffi::c_void, *const u16, u32) -> windows::core::HRESULT;
    type ReleaseFn = unsafe extern "system" fn(*mut std::ffi::c_void) -> u32;

    let set_default: SetDefaultEndpointFn = std::mem::transmute(*vtable.add(13));
    let release: ReleaseFn = std::mem::transmute(*vtable.add(2));

    let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();

    // eConsole = 0, eMultimedia = 1, eCommunications = 2
    for role in 0..3u32 {
        let _ = set_default(obj, wide.as_ptr(), role);
    }

    release(obj);
    Ok(())
}

#[cfg(windows)]
unsafe fn get_device_name(device: &IMMDevice) -> Option<String> {
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;

    let store: IPropertyStore = device.OpenPropertyStore(STGM(0)).ok()?;

    // PKEY_Device_FriendlyName
    let key = PROPERTYKEY {
        fmtid: windows::core::GUID::from_values(
            0xa45c254e,
            0xdf1c,
            0x4efd,
            [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0],
        ),
        pid: 14,
    };

    let value = store.GetValue(&key).ok()?;

    // VT_LPWSTR = 31; extract the wide string pointer from the PROPVARIANT union
    let raw = &*(&value as *const _ as *const PROPVARIANT_RAW);
    if raw.vt != 31 {
        return None;
    }
    let pwstr = raw.val as *const u16;
    if pwstr.is_null() {
        return None;
    }
    let len = (0..).take_while(|&i| *pwstr.add(i) != 0).count();
    let slice = std::slice::from_raw_parts(pwstr, len);
    Some(String::from_utf16_lossy(slice))
}

#[cfg(windows)]
#[repr(C)]
struct PROPVARIANT_RAW {
    vt: u16,
    _pad: [u16; 3],
    val: usize,
}

#[cfg(windows)]
pub fn get_audio_processes() -> Vec<ProcessInfo> {
    let mut result = Vec::new();

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let Ok(enumerator) =
            CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
        else {
            return result;
        };

        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else {
            return result;
        };

        let Ok(session_mgr): Result<IAudioSessionManager2, _> = device.Activate(CLSCTX_ALL, None)
        else {
            return result;
        };

        let Ok(session_enum) = session_mgr.GetSessionEnumerator() else {
            return result;
        };

        let Ok(count) = session_enum.GetCount() else {
            return result;
        };

        let mut seen = std::collections::HashSet::new();

        for i in 0..count {
            let Ok(session) = session_enum.GetSession(i) else {
                continue;
            };
            let Ok(session2): Result<IAudioSessionControl2, _> = session.cast() else {
                continue;
            };
            let Ok(pid) = session2.GetProcessId() else {
                continue;
            };
            if pid == 0 || !seen.insert(pid) {
                continue;
            }

            let name = get_process_name(pid).unwrap_or_else(|| format!("PID {}", pid));
            result.push(ProcessInfo { pid, name });
        }
    }

    result
}

#[cfg(not(windows))]
pub fn get_audio_processes() -> Vec<ProcessInfo> {
    Vec::new()
}

#[cfg(windows)]
fn get_process_name(pid: u32) -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 260];
        let mut size = buf.len() as u32;

        let ok = windows::Win32::System::Threading::QueryFullProcessImageNameW(
            handle,
            windows::Win32::System::Threading::PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        );

        let _ = CloseHandle(handle);

        if ok.is_ok() {
            let path = String::from_utf16_lossy(&buf[..size as usize]);
            let filename = std::path::Path::new(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(path);
            Some(filename)
        } else {
            None
        }
    }
}

fn run_loop(tx: &flume::Sender<AudioEvent>, running: &Arc<AtomicBool>, cfg: &StreamConfig) {
    while running.load(Ordering::Acquire) {
        let _ = tx.send(AudioEvent::StateChanged(StreamState::Connecting));
        let _ = tx.send(AudioEvent::Log(format!(
            "Connecting to {}:{}...",
            cfg.server, cfg.port
        )));

        match do_stream(tx, running, cfg) {
            Ok(()) => {}
            Err(e) => {
                if !running.load(Ordering::Acquire) {
                    break;
                }
                let _ = tx.send(AudioEvent::Log(format!("Error: {}", e)));
                let _ = tx.send(AudioEvent::StateChanged(StreamState::Disconnected));
                let _ = tx.send(AudioEvent::Log("Reconnecting in 3s...".to_string()));

                for _ in 0..30 {
                    if !running.load(Ordering::Acquire) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    let _ = tx.send(AudioEvent::StateChanged(StreamState::Disconnected));
}

#[cfg(windows)]
#[windows::core::implement(IActivateAudioInterfaceCompletionHandler)]
struct ActivationHandler {
    event: Arc<(Mutex<bool>, Condvar)>,
}

#[cfg(windows)]
impl IActivateAudioInterfaceCompletionHandler_Impl for ActivationHandler_Impl {
    fn ActivateCompleted(
        &self,
        _op: windows_core::Ref<'_, IActivateAudioInterfaceAsyncOperation>,
    ) -> windows::core::Result<()> {
        let (lock, cvar) = &*self.event;
        *lock.lock().unwrap() = true;
        cvar.notify_one();
        Ok(())
    }
}

#[cfg(windows)]
struct VolumeState {
    volume: f32,
    muted: bool,
}

#[cfg(windows)]
#[windows::core::implement(IAudioEndpointVolumeCallback)]
struct VolumeCallback {
    state: Arc<Mutex<VolumeState>>,
    mute_local_output: bool,
    ep_vol: IAudioEndpointVolume,
    tx: flume::Sender<AudioEvent>,
}

#[cfg(windows)]
impl IAudioEndpointVolumeCallback_Impl for VolumeCallback_Impl {
    fn OnNotify(&self, data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        if data.is_null() {
            return Ok(());
        }
        unsafe {
            let new_vol = (*data).fMasterVolume;
            let new_mute = (*data).bMuted.as_bool();

            if self.mute_local_output && !new_mute {
                let _ = self.ep_vol.SetMute(true, std::ptr::null());
            }

            if let Ok(mut st) = self.state.lock() {
                st.volume = new_vol;
                st.muted = new_mute;
            }

            let _ = self.tx.send(AudioEvent::VolumeChanged {
                volume: new_vol,
                muted: if self.mute_local_output {
                    true
                } else {
                    new_mute
                },
            });
        }
        Ok(())
    }
}

#[cfg(windows)]
unsafe fn activate_process_loopback(pid: u32) -> Result<IAudioClient, Box<dyn std::error::Error>> {
    let params = AUDIOCLIENT_ACTIVATION_PARAMS {
        ActivationType: AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK,
        Anonymous: AUDIOCLIENT_ACTIVATION_PARAMS_0 {
            ProcessLoopbackParams: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS {
                TargetProcessId: pid,
                ProcessLoopbackMode: PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE,
            },
        },
    };

    // Build PROPVARIANT with VT_BLOB pointing to activation params
    #[repr(C)]
    struct BlobPropVariant {
        vt: u16,
        pad: [u16; 3],
        cb_size: u32,
        _align: u32,
        p_data: *const u8,
    }

    let pv = BlobPropVariant {
        vt: 0x0041, // VT_BLOB
        pad: [0; 3],
        cb_size: std::mem::size_of::<AUDIOCLIENT_ACTIVATION_PARAMS>() as u32,
        _align: 0,
        p_data: &params as *const AUDIOCLIENT_ACTIVATION_PARAMS as *const u8,
    };

    let riid = IAudioClient::IID;

    let setup = Arc::new((Mutex::new(false), Condvar::new()));
    let callback: IActivateAudioInterfaceCompletionHandler = ActivationHandler {
        event: setup.clone(),
    }
    .into();

    let operation = ActivateAudioInterfaceAsync(
        VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK,
        &riid,
        Some(&pv as *const BlobPropVariant as *const _),
        &callback,
    )?;

    let (lock, cvar) = &*setup;
    let mut completed = lock.lock().unwrap();
    while !*completed {
        let (c, timeout) = cvar
            .wait_timeout(completed, Duration::from_secs(5))
            .unwrap();
        completed = c;
        if timeout.timed_out() {
            return Err("Process loopback activation timed out".into());
        }
    }
    drop(completed);

    let mut hr = windows::core::HRESULT(0);
    let mut activated: Option<windows::core::IUnknown> = None;
    operation.GetActivateResult(&mut hr, &mut activated)?;
    hr.ok()?;

    let client: IAudioClient = activated
        .ok_or("Activation returned null audio client")?
        .cast()?;

    Ok(client)
}

#[cfg(windows)]
fn do_stream(
    tx: &flume::Sender<AudioEvent>,
    running: &Arc<AtomicBool>,
    cfg: &StreamConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mute_local_output = cfg.mute_local_output;
    let process_id = cfg.process_id;
    let tcp = TcpStream::connect(format!("{}:{}", cfg.server, cfg.port))?;
    tcp.set_nodelay(true)?;
    tcp.set_write_timeout(Some(Duration::from_secs(3)))?;
    // Send buffer: enough to absorb brief TCP delays without dropping chunks
    set_send_buffer(&tcp, 8192);
    set_tcp_keepalive(&tcp);

    // Writer thread: capture never blocks on TCP; when dialog opens and write blocks, we drop chunks instead of stalling
    let (tx_chunk, rx_chunk) = flume::bounded::<Arc<[u8]>>(16);
    let writer_handle = std::thread::spawn(move || {
        let mut stream = tcp;
        while let Ok(ref buf) = rx_chunk.recv() {
            if stream.write_all(buf).is_err() {
                break;
            }
        }
    });

    let _ = tx.send(AudioEvent::StateChanged(StreamState::Connected));
    let _ = tx.send(AudioEvent::Log(format!(
        "Connected to {}:{}",
        cfg.server, cfg.port
    )));

    let mut connection_lost = false;
    let is_vbcable = cfg.capture_mode == CaptureMode::VbCable;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        let mut ep_vol: Option<IAudioEndpointVolume> = None;
        let mut vol_callback: Option<IAudioEndpointVolumeCallback> = None;
        let mut original_mute = false;
        let mut original_default_id: Option<String> = None;
        let vol_state = Arc::new(Mutex::new(VolumeState {
            volume: 1.0,
            muted: false,
        }));

        if is_vbcable {
            let _ = tx.send(AudioEvent::Log("VB-CABLE mode".to_string()));

            // Find VB-CABLE render device and switch Windows default output to it
            if let Some(render_dev) = detect_vb_cable_render() {
                original_default_id = get_default_endpoint_id(&enumerator, eMultimedia);

                let current_default = original_default_id.as_deref().unwrap_or("");
                if current_default != render_dev.id {
                    let _ = tx.send(AudioEvent::Log(format!(
                        "Switching output to: {}",
                        render_dev.name
                    )));
                    let _ = set_default_endpoint(&render_dev.id);
                } else {
                    let _ = tx.send(AudioEvent::Log(
                        "VB-CABLE already default output".to_string(),
                    ));
                    original_default_id = None;
                }

                // Monitor volume on the VB-CABLE render endpoint
                let wide: Vec<u16> = render_dev
                    .id
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                if let Ok(render_device) =
                    enumerator.GetDevice(windows::core::PCWSTR(wide.as_ptr()))
                {
                    if let Ok(vol) =
                        render_device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                    {
                        let init_volume = vol.GetMasterVolumeLevelScalar().unwrap_or(1.0);
                        let init_mute = vol.GetMute().map(|b| b.as_bool()).unwrap_or(false);

                        *vol_state.lock().unwrap() = VolumeState {
                            volume: init_volume,
                            muted: init_mute,
                        };

                        let cb: IAudioEndpointVolumeCallback = VolumeCallback {
                            state: vol_state.clone(),
                            mute_local_output: false,
                            ep_vol: vol.clone(),
                            tx: tx.clone(),
                        }
                        .into();
                        let _ = vol.RegisterControlChangeNotify(&cb);

                        let _ = tx.send(AudioEvent::VolumeChanged {
                            volume: init_volume,
                            muted: init_mute,
                        });

                        vol_callback = Some(cb);
                        ep_vol = Some(vol);
                    }
                }
            }
        } else {
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
            let vol: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            let init_volume = vol.GetMasterVolumeLevelScalar()?;
            let init_mute = vol.GetMute()?.as_bool();

            original_mute = init_mute;
            if mute_local_output && !init_mute {
                let _ = vol.SetMute(true, std::ptr::null());
            }

            *vol_state.lock().unwrap() = VolumeState {
                volume: init_volume,
                muted: if mute_local_output { true } else { init_mute },
            };

            let cb: IAudioEndpointVolumeCallback = VolumeCallback {
                state: vol_state.clone(),
                mute_local_output,
                ep_vol: vol.clone(),
                tx: tx.clone(),
            }
            .into();
            vol.RegisterControlChangeNotify(&cb)?;

            let _ = tx.send(AudioEvent::VolumeChanged {
                volume: init_volume,
                muted: if mute_local_output { true } else { init_mute },
            });

            vol_callback = Some(cb);
            ep_vol = Some(vol);
        }

        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL);

        let buffer_duration: i64 = 100_000;

        let event_handle = CreateEventW(None, false, false, windows::core::PCWSTR::null())?;

        let (client, src_rate_val, src_ch_val, src_bits_val, is_float) = if is_vbcable {
            let vb_device_id = cfg.device_id.as_ref().ok_or("VB-CABLE device ID not set")?;
            let wide: Vec<u16> = vb_device_id
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let device = enumerator.GetDevice(windows::core::PCWSTR(wide.as_ptr()))?;
            let _ = tx.send(AudioEvent::Log(format!(
                "Capturing from VB-CABLE: {}",
                get_device_name(&device).unwrap_or_default()
            )));

            let client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
            let mix_format_ptr = client.GetMixFormat()?;

            let src_rate_val = (*mix_format_ptr).nSamplesPerSec;
            let src_ch_val = (*mix_format_ptr).nChannels;
            let src_bits_val = (*mix_format_ptr).wBitsPerSample;

            client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                buffer_duration,
                0,
                &*mix_format_ptr,
                None,
            )?;

            let fmt_tag = (*mix_format_ptr).wFormatTag;
            let is_float = fmt_tag == 3 || (fmt_tag == 0xFFFE && src_bits_val == 32);
            CoTaskMemFree(Some(mix_format_ptr as *const _));

            (client, src_rate_val, src_ch_val, src_bits_val, is_float)
        } else if let Some(pid) = process_id {
            let _ = tx.send(AudioEvent::Log(format!("Per-app capture: PID {}", pid)));
            let client = activate_process_loopback(pid)?;

            let mut fmt: WAVEFORMATEX = std::mem::zeroed();
            fmt.wFormatTag = 3; // WAVE_FORMAT_IEEE_FLOAT
            fmt.nChannels = 2;
            fmt.nSamplesPerSec = 48000;
            fmt.wBitsPerSample = 32;
            fmt.nBlockAlign = 8;
            fmt.nAvgBytesPerSec = 384000;
            fmt.cbSize = 0;

            const AUTOCONVERTPCM: u32 = 0x80000000;
            const SRC_DEFAULT_QUALITY: u32 = 0x08000000;

            client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK
                    | AUDCLNT_STREAMFLAGS_EVENTCALLBACK
                    | AUTOCONVERTPCM
                    | SRC_DEFAULT_QUALITY,
                buffer_duration,
                0,
                &fmt,
                None,
            )?;

            (client, 48000u32, 2u16, 32u16, true)
        } else {
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
            let client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
            let mix_format_ptr = client.GetMixFormat()?;

            let src_rate_val = (*mix_format_ptr).nSamplesPerSec;
            let src_ch_val = (*mix_format_ptr).nChannels;
            let src_bits_val = (*mix_format_ptr).wBitsPerSample;

            client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                buffer_duration,
                0,
                &*mix_format_ptr,
                None,
            )?;

            let fmt_tag = (*mix_format_ptr).wFormatTag;
            let is_float = fmt_tag == 3 || (fmt_tag == 0xFFFE && src_bits_val == 32);
            CoTaskMemFree(Some(mix_format_ptr as *const _));

            (client, src_rate_val, src_ch_val, src_bits_val, is_float)
        };

        let format_str = format!(
            "{:.1}kHz {}ch {}bit",
            src_rate_val as f64 / 1000.0,
            src_ch_val,
            src_bits_val
        );
        let _ = tx.send(AudioEvent::Log(format!("Capture: {}", format_str)));

        client.SetEventHandle(event_handle)?;

        let capture: IAudioCaptureClient = client.GetService()?;

        client.Start()?;

        let _ = tx.send(AudioEvent::StateChanged(StreamState::Streaming));
        let _ = tx.send(AudioEvent::Log("Streaming".to_string()));

        let start = Instant::now();
        let mut total_bytes: u64 = 0;
        let mut prev_bytes: u64 = 0;
        let mut prev_time = Instant::now();
        let mut avg_latency_ms: f64 = 0.0;
        let mut drop_count: u32 = 0;
        let capture_buffer_ms: f64 = buffer_duration as f64 / 10_000.0;

        let src_channels = src_ch_val as usize;

        // Pre-allocate reusable buffers to avoid per-frame heap allocations
        let mut float_buf: Vec<f32> = Vec::with_capacity(src_rate_val as usize / 25 * src_channels);
        let mut pcm_buf: Vec<u8> = Vec::with_capacity(float_buf.capacity() * 2);

        let silence_10ms_len = (src_rate_val as usize * src_channels * 2) / 100;
        let silence_chunk: Arc<[u8]> = vec![0u8; silence_10ms_len].into();

        while running.load(Ordering::Acquire) {
            // Writer thread exits when TCP fails; capture must stop so run_loop can reconnect.
            if writer_handle.is_finished() {
                connection_lost = true;
                break;
            }

            let wait_result = WaitForSingleObject(event_handle, 100);
            if wait_result != WAIT_OBJECT_0 {
                let _ = tx_chunk.try_send(Arc::clone(&silence_chunk));
            }
            if wait_result == WAIT_OBJECT_0 {
                loop {
                    if writer_handle.is_finished() {
                        connection_lost = true;
                        break;
                    }

                    let mut buffer_ptr = std::ptr::null_mut();
                    let mut num_frames = 0u32;
                    let mut flags = 0u32;

                    let hr =
                        capture.GetBuffer(&mut buffer_ptr, &mut num_frames, &mut flags, None, None);

                    if hr.is_err() || num_frames == 0 {
                        break;
                    }

                    let capture_instant = Instant::now();
                    let sample_count = num_frames as usize * src_channels;

                    float_buf.clear();
                    if is_float {
                        let floats =
                            std::slice::from_raw_parts(buffer_ptr as *const f32, sample_count);
                        float_buf.extend_from_slice(floats);
                    } else {
                        let shorts =
                            std::slice::from_raw_parts(buffer_ptr as *const i16, sample_count);
                        float_buf.extend(shorts.iter().map(|&s| s as f32 / 32768.0));
                    }

                    let _ = capture.ReleaseBuffer(num_frames);

                    let vol = {
                        let st = vol_state.lock().unwrap();
                        if !is_vbcable && mute_local_output {
                            st.volume.max(0.01)
                        } else if st.muted {
                            0.0f32
                        } else {
                            st.volume
                        }
                    };
                    let byte_len = sample_count * 2;
                    pcm_buf.clear();
                    if pcm_buf.capacity() < byte_len {
                        pcm_buf.reserve(byte_len - pcm_buf.capacity());
                    }
                    pcm_buf.set_len(byte_len);
                    let pcm_i16 = std::slice::from_raw_parts_mut(
                        pcm_buf.as_mut_ptr() as *mut i16,
                        sample_count,
                    );
                    for (out, &s) in pcm_i16.iter_mut().zip(float_buf.iter()) {
                        *out = ((s * vol).clamp(-1.0, 1.0) * 32767.0) as i16;
                    }

                    let chunk: Arc<[u8]> = pcm_buf.clone().into();
                    if tx_chunk.try_send(chunk).is_ok() {
                        total_bytes += pcm_buf.len() as u64;
                    } else {
                        drop_count += 1;
                    }

                    let send_elapsed_ms = capture_instant.elapsed().as_secs_f64() * 1000.0;
                    let chunk_latency = capture_buffer_ms + send_elapsed_ms;
                    avg_latency_ms = avg_latency_ms * 0.8 + chunk_latency * 0.2;
                }
                if connection_lost {
                    break;
                }
            }

            let now = Instant::now();
            let elapsed_ms = now.duration_since(prev_time).as_millis() as u64;
            if elapsed_ms >= 500 {
                let kbps = (total_bytes - prev_bytes) as f64 * 8.0 / elapsed_ms as f64;
                let _ = tx.send(AudioEvent::StatsUpdated(Stats {
                    bytes_sent: total_bytes,
                    bitrate_kbps: kbps,
                    uptime: start.elapsed(),
                    client_latency_ms: avg_latency_ms,
                    drops: drop_count,
                    capture_format: format_str.clone(),
                }));
                prev_bytes = total_bytes;
                prev_time = now;
            }
        }

        drop(tx_chunk);
        let _ = writer_handle.join();

        client.Stop()?;
        let _ = CloseHandle(event_handle);

        if let (Some(ref vol), Some(ref cb)) = (&ep_vol, &vol_callback) {
            let _ = vol.UnregisterControlChangeNotify(cb);
        }

        if !is_vbcable && mute_local_output && !original_mute {
            if let Some(ref vol) = ep_vol {
                let _ = vol.SetMute(false, std::ptr::null());
            }
        }

        if let Some(ref orig_id) = original_default_id {
            let _ = tx.send(AudioEvent::Log(
                "Restoring original output device".to_string(),
            ));
            let _ = set_default_endpoint(orig_id);
        }
    }

    if connection_lost {
        // run_loop logs the error and emits Disconnected + reconnect delay
        return Err("connection lost".into());
    }

    let _ = tx.send(AudioEvent::StateChanged(StreamState::Disconnected));
    let _ = tx.send(AudioEvent::Log("Disconnected".to_string()));
    Ok(())
}

#[cfg(not(windows))]
fn do_stream(
    tx: &flume::Sender<AudioEvent>,
    _running: &Arc<AtomicBool>,
    _cfg: &StreamConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = tx.send(AudioEvent::Log(
        "WASAPI only available on Windows".to_string(),
    ));
    Err("unsupported platform".into())
}

#[cfg(windows)]
fn set_send_buffer(tcp: &TcpStream, size: u32) {
    use std::os::windows::io::AsRawSocket;
    extern "system" {
        fn setsockopt(s: usize, level: i32, optname: i32, optval: *const u8, optlen: i32) -> i32;
    }
    let raw = tcp.as_raw_socket();
    let val = size as i32;
    unsafe {
        setsockopt(
            raw as usize,
            0xFFFF,
            0x1001,
            &val as *const i32 as *const u8,
            4,
        );
    }
}

#[cfg(not(windows))]
fn set_send_buffer(_tcp: &TcpStream, _size: u32) {}

/// Enable TCP keepalive so the stack probes dead peers (router/NAT drops) instead of hanging forever.
#[cfg(windows)]
fn set_tcp_keepalive(tcp: &TcpStream) {
    use std::os::windows::io::AsRawSocket;
    extern "system" {
        fn setsockopt(s: usize, level: i32, optname: i32, optval: *const u8, optlen: i32) -> i32;
    }
    const SO_KEEPALIVE: i32 = 0x0008;
    let raw = tcp.as_raw_socket();
    let on: u32 = 1;
    unsafe {
        setsockopt(
            raw as usize,
            0xFFFF,
            SO_KEEPALIVE,
            &on as *const u32 as *const u8,
            4,
        );
    }
}

#[cfg(not(windows))]
fn set_tcp_keepalive(_tcp: &TcpStream) {}
