use crate::manifest::Manifest;
use crate::hilbert::tree::HilbertTree;

pub fn generate(manifest: &Manifest) -> Result<(), Box<dyn std::error::Error>> {
    let report_path = manifest.data.planet.join("report.yaml");
    println!("Generating report at: {}", report_path.display());

    let tree = HilbertTree::open(manifest)?;

    let leaf_it = tree.pvt_leaf_iterator();

    for (tile, buffer) in leaf_it {
        let size = buffer.len();
        
        println!("{} {} bytes", tile, size);
    }

    Ok(())
}
