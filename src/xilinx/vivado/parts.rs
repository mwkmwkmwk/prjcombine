use crate::toolchain::Toolchain;
use crate::toolreader::ToolchainReader;
use crate::error::Error;
use std::io::BufRead;

#[derive(Debug)]
pub struct VivadoPart {
    pub name: String,
    pub actual_family: String,
    pub arch: String,
    pub family: String,
    pub device: String,
    pub package: String,
    pub speed: String,
    pub temp: String,
}

const GET_PARTS_TCL: &str = r#"
set fd [open "parts.fifo" w]
foreach x [get_parts] {
    set arch [get_property ARCHITECTURE $x]
    set fam [get_property FAMILY $x]
    set dev [get_property DEVICE $x]
    set pkg [get_property PACKAGE $x]
    set speed [get_property SPEED $x]
    set temp [get_property TEMPERATURE_GRADE_LETTER $x]
    puts $fd "PART $x $arch $fam $dev $pkg $speed $temp"
}
puts $fd "END"
"#;

pub fn get_parts(tc: &Toolchain) -> Result<Vec<VivadoPart>, Error> {
    let tr = ToolchainReader::new(tc, "vivado", &["-nolog", "-nojournal", "-mode", "batch", "-source", "script.tcl"], &[], "parts.fifo", &[("script.tcl", GET_PARTS_TCL.as_bytes())])?;
    let lines = tr.lines();
    let mut res: Vec<VivadoPart> = Vec::new();
    let mut got_end = false;
    for l in lines {
        let l = l?;
        let sl: Vec<_> = l.split_whitespace().collect();
        if sl[0] == "END" {
            got_end = true;
            break;
        }
        assert!(sl[0] == "PART");
        res.push(VivadoPart{
            name: sl[1].to_string(),
            actual_family: match sl[2] {
                "spartan7" => "7series",
                "artix7" => "7series",
                "kintex7" => "7series",
                "virtex7" => "7series",
                "zynq" => "7series",
                "kintexu" => "ultrascale",
                "virtexu" => "ultrascale",
                "kintexuplus" => "ultrascaleplus",
                "virtexuplus" => "ultrascaleplus",
                "virtexuplusHBM" => "ultrascaleplus",
                "virtexuplus58g" => "ultrascaleplus",
                "zynquplus" => "ultrascaleplus",
                "zynquplusRFSOC" => "ultrascaleplus",
                "versal" => "versal",
                _ => panic!("unknown arch {}", sl[2]),
            }.to_string(),
            arch: sl[2].to_string(),
            family: sl[3].to_string(),
            device: sl[4].to_string(),
            package: sl[5].to_string(),
            speed: sl[6].to_string(),
            temp: sl.get(7).unwrap_or(&"").to_string(),
        });
    }
    if !got_end {
        return Err(Error::ParseError(format!("missing END")));
    }
    Ok(res)
}
