use ifcfg;

fn main() -> ifcfg::Result<()> {
    let ifaces = ifcfg::IfCfg::get().expect("could not get interfaces");
    println!("{:#?}", &ifaces);
    Ok(())
}