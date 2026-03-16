fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "PulseStream");
        res.set("FileDescription", "Stream Windows audio to PulseAudio");
        let _ = res.compile();
    }
}
