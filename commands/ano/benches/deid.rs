use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use pulsedcm_core::*;
use pulsedcm_commands_ano::threading_handling;
use std::path::PathBuf;
use std::env;

fn criterion_bench(c: &mut Criterion){
    let bpath = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("pulsedcm-utils/benchmark/data");
    // println!("{:?}",PathBuf::from(bpath.parent().unwrap().parent().unwrap().join("benchmark/crit_bench/mg_bench")));
    let mg_data = collect_dicom_files(bpath.join("1_one_big_image_MG").to_str().unwrap()).unwrap();
    let mr_data = collect_dicom_files(bpath.join("2_small_serie_MR").to_str().unwrap()).unwrap();
    let ct_data = collect_dicom_files(bpath.join("3_big_serie_CT").to_str().unwrap()).unwrap();

    c.bench_function("MG", |b| b.iter(|| {
        let mut b = false;
        let uid_to_hash = false;
        threading_handling(
            mg_data.clone(), 
            PathBuf::from(bpath.parent().unwrap().parent().unwrap().join("benchmark/crit_bench/mg_bench")),
            &mut b,
            true, 
            0,
            1,
            false,
            &uid_to_hash,
        )
    }));
    c.bench_function("MR_1", |b| b.iter(|| {
        let mut b = false;
        let uid_to_hash = false;
        threading_handling(
            mr_data.clone(), 
            PathBuf::from(bpath.parent().unwrap().parent().unwrap().join("benchmark/crit_bench/mr_bench")),
            &mut b,
            true, 
            0,
            1,
            false,
            &uid_to_hash,
        )
    }));
    c.bench_function("MR_25", |b| b.iter(|| {
        let mut b = false;
        let uid_to_hash = false;
        threading_handling(
            mr_data.clone(), 
            PathBuf::from(bpath.parent().unwrap().parent().unwrap().join("benchmark/crit_bench/mr_bench")),
            &mut b,
            true, 
            0,
            25,
            false,
            &uid_to_hash,
        )
    }));

    c.bench_function("CT_1", |b| b.iter(|| {
        let mut b = false;
        let uid_to_hash = false;
        threading_handling(
            ct_data.clone(), 
            PathBuf::from(bpath.parent().unwrap().parent().unwrap().join("benchmark/crit_bench/ct_bench")),
            &mut b,
            true, 
            0,
            1,
            false,
            &uid_to_hash,
        )
    }));
    c.bench_function("CT_25", |b| b.iter(|| {
        let mut b = false;
        let uid_to_hash = false;
        threading_handling(
            ct_data.clone(), 
            PathBuf::from(bpath.parent().unwrap().parent().unwrap().join("benchmark/crit_bench/ct_bench")),
            &mut b,
            true, 
            0,
            25,
            false,
            &uid_to_hash,
        )
    }));
    c.bench_function("CT_75", |b| b.iter(|| {
        let mut b = false;
        let uid_to_hash = false;
        threading_handling(
            ct_data.clone(), 
            PathBuf::from(bpath.parent().unwrap().parent().unwrap().join("benchmark/crit_bench/ct_bench")),
            &mut b,
            true, 
            0,
            75,
            false,
            &uid_to_hash,
        )
    }));
}



criterion_group!(benches, criterion_bench);
criterion_main!(benches);
