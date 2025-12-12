use fat32_rust::{Fat32, std_support::StdBlockDevice};

#[test]
fn list_root_contains_readme_and_dir1() {
    let dev = StdBlockDevice::open("images/test_fat32.img", 512).unwrap();
    let mut fs = Fat32::new(dev).unwrap();

    let entries = fs.list_root().unwrap();

    assert!(entries.iter().any(|e| e.name == "README.TXT" && !e.is_dir));
    assert!(entries.iter().any(|e| e.name == "DIR1" && e.is_dir));
}

#[test]
fn read_root_readme() {
    let dev = StdBlockDevice::open("images/test_fat32.img", 512).unwrap();
    let mut fs = Fat32::new(dev).unwrap();

    let data = fs.read_file("/README.TXT").unwrap();
    let s = String::from_utf8_lossy(&data);

    assert!(s.contains("Hello from FAT32 root"));
}

#[test]
fn cd_and_relative_read() {
    let dev = StdBlockDevice::open("images/test_fat32.img", 512).unwrap();
    let mut fs = Fat32::new(dev).unwrap();

    fs.change_dir("/DIR1").unwrap();
    let data = fs.read_file("FILE1.TXT").unwrap();
    let s = String::from_utf8_lossy(&data);

    assert!(s.contains("Ceci est un fichier dans DIR1"));
}

#[test]
fn cd_dotdot_goes_back_to_root() {
    let dev = StdBlockDevice::open("images/test_fat32.img", 512).unwrap();
    let mut fs = Fat32::new(dev).unwrap();

    fs.change_dir("/DIR1").unwrap();
    fs.change_dir("..").unwrap();

    let entries = fs.list_cwd().unwrap();
    assert!(entries.iter().any(|e| e.name == "README.TXT"));
}

#[test]
fn ls_cwd_after_cd_dir1_contains_file1() {
    let dev = StdBlockDevice::open("images/test_fat32.img", 512).unwrap();
    let mut fs = Fat32::new(dev).unwrap();

    fs.change_dir("/DIR1").unwrap();
    let entries = fs.list_cwd().unwrap();

    assert!(entries.iter().any(|e| e.name == "FILE1.TXT" && !e.is_dir));
}
