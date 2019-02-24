extern crate winres;
extern crate git_version;

use failure::Error;

fn main() -> Result<(), Error> {
    git_version::set_env();

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/icon256.ico");
        res.compile()?;
    }
    Ok(())
}
