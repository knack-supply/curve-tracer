extern crate winres;

use failure::Error;

fn main() -> Result<(), Error> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon256.ico");
        res.compile()?;
    }
    Ok(())
}
