use std::io;
use std::path::PathBuf;
use std::fs::create_dir_all;
use std::collections::HashMap;
use structopt::StructOpt;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use prjcombine::xilinx::ise::rawdump::get_rawdump;
use prjcombine::xilinx::ise::partgen::{get_pkgs, PartgenPkg};
use prjcombine::toolchain::Toolchain;

#[derive(Debug, StructOpt)]
#[structopt(name = "dump_ise_parts", about = "Dump ISE part geometry into rawdump files.")]
struct Opt {
    toolchain: String,
    #[structopt(parse(from_os_str))]
    target_directory: PathBuf,
    families: Vec<String>,
    #[structopt(short="n", long, default_value="0")]
    num_threads: usize,
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();
    ThreadPoolBuilder::new().num_threads(opt.num_threads).build_global().unwrap();
    let tc = Toolchain::from_file(&opt.toolchain)?;
    let mut ise_families: Vec<&'static str> = Vec::new();
    for family in opt.families.iter() {
        ise_families.extend(match &family[..] {
            "xc4000e" => vec!["xc4000e", "xc4000l", "spartan"],
            "xc4000ex" => vec!["xc4000ex", "xc4000xl"],
            "xc4000xla" => vec!["xc4000xla"],
            "xc4000xv" => vec!["xc4000xv"],
            "spartanxl" => vec!["spartanxl"],
            "virtex" => vec!["virtex", "qvirtex", "qrvirtex", "spartan2"],
            "virtexe" => vec!["virtexe", "qvirtexe", "spartan2e", "aspartan2e"],
            "virtex2" => vec!["virtex2", "qvirtex2", "qrvirtex2"],
            "virtex2p" => vec!["virtex2p", "qvirtex2p"],
            "spartan3" => vec!["spartan3", "aspartan3"],
            "spartan3e" => vec!["spartan3e", "aspartan3e"],
            "spartan3a" => vec!["spartan3a", "aspartan3a"],
            "spartan3adsp" => vec!["spartan3adsp", "aspartan3adsp"],
            "spartan6" => vec!["spartan6", "spartan6l", "aspartan6", "qspartan6", "qspartan6l"],
            "virtex4" => vec!["virtex4", "qvirtex4", "qrvirtex4"],
            "virtex5" => vec!["virtex5", "qvirtex5"],
            "virtex6" => vec!["virtex6", "virtex6l", "qvirtex6", "qvirtex6l"],
            "7series" => vec![
                "artix7", "artix7l", "aartix7", "qartix7",
                "kintex7", "kintex7l", "qkintex7", "qkintex7l",
                "virtex7", "qvirtex7",
                "zynq", "azynq", "qzynq",
            ],
            _ => return Err(io::Error::new(io::ErrorKind::Other, format!("unknown family {}", family))),
        });
    };
    create_dir_all(&opt.target_directory)?;
    let mut parts: HashMap<String, Vec<PartgenPkg>> = HashMap::new();
    for ise_fam in ise_families.iter() {
        println!("querying {}", ise_fam);
    }
    let pkg_list: Vec<_> = ise_families.into_par_iter().map(|ise_fam| get_pkgs(&tc, ise_fam)).collect();
    for pkgs in pkg_list {
        for pkg in pkgs? {
            match parts.get_mut(&pkg.device) {
                None => { parts.insert(pkg.device.to_string(), vec![pkg]); },
                Some(v) => { v.push(pkg); },
            }
        }
    }
    for (part, pkgs) in parts.iter() {
        println!("device {} [{}]: {}", part, pkgs[0].family, pkgs.iter().fold(String::new(), |acc, pkg| acc + &pkg.package + ", "));
    }
    for res in parts.into_par_iter().map(|(part, pkgs)| -> Result<(), io::Error> {
        let fdir = opt.target_directory.join(&pkgs[0].family);
        create_dir_all(&fdir)?;
        let path = fdir.join(part.clone() + ".zstd");
        if path.exists() {
            println!("skipping {}", part);
        } else {
            println!("dumping {}", part);
            let rd = get_rawdump(&tc, &pkgs)?;
            rd.to_file(&path)?;
            println!("dumped {}", part);
        }
        Ok(())
    }).collect::<Vec<_>>() {
        res?;
    }
    Ok(())
}
