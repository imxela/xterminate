extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("res/icon.ico");

    res.set_manifest(r#"
        <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
            <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
                <security>
                    <requestedPrivileges>
                        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
                    </requestedPrivileges>
                </security>
            </trustInfo>
        </assembly>
    "#);

    // English = 0x0409
    res.set_language(0x0409);

    res.compile().unwrap();
}