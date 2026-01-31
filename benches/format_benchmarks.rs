use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ebook_cli::formats::{EpubHandler, CbzHandler, TxtHandler, PdfHandler};
use ebook_cli::traits::{EbookReader, EbookWriter};
use ebook_cli::Metadata;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_epub(size: usize) -> PathBuf {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("bench.epub");
    
    let mut handler = EpubHandler::new();
    handler.set_metadata(Metadata::new().with_title("Benchmark Test")).unwrap();
    
    for i in 0..size {
        let content = format!("<h1>Chapter {}</h1><p>{}</p>", i, "Lorem ipsum ".repeat(100));
        handler.add_chapter(&format!("Chapter {}", i), &content).unwrap();
    }
    
    handler.write_to_file(&path).unwrap();
    
    // Keep temp_dir alive by leaking it
    std::mem::forget(temp_dir);
    path
}

fn create_test_cbz(size: usize) -> PathBuf {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("bench.cbz");
    
    let mut handler = CbzHandler::new();
    handler.set_metadata(Metadata::new().with_title("Benchmark Comic")).unwrap();
    
    let test_image = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    
    for i in 0..size {
        handler.add_image(&format!("page{:03}.png", i), test_image.clone()).unwrap();
    }
    
    handler.write_to_file(&path).unwrap();
    
    std::mem::forget(temp_dir);
    path
}

fn bench_epub_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("epub_write");
    
    for size in [1, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let path = temp_dir.path().join("test.epub");
                
                let mut handler = EpubHandler::new();
                handler.set_metadata(Metadata::new().with_title("Test")).unwrap();
                
                for i in 0..size {
                    handler.add_chapter(&format!("Ch{}", i), "<h1>Test</h1>").unwrap();
                }
                
                handler.write_to_file(&path).unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_epub_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("epub_read");
    
    for size in [1, 10, 50].iter() {
        let path = create_test_epub(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut handler = EpubHandler::new();
                handler.read_from_file(black_box(&path)).unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_cbz_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("cbz_write");
    
    for size in [1, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let path = temp_dir.path().join("test.cbz");
                
                let mut handler = CbzHandler::new();
                handler.set_metadata(Metadata::new().with_title("Test")).unwrap();
                
                let test_image = vec![0x89, 0x50, 0x4E, 0x47];
                for i in 0..size {
                    handler.add_image(&format!("p{}.png", i), test_image.clone()).unwrap();
                }
                
                handler.write_to_file(&path).unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_cbz_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("cbz_read");
    
    for size in [1, 10, 50].iter() {
        let path = create_test_cbz(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut handler = CbzHandler::new();
                handler.read_from_file(black_box(&path)).unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_metadata_extraction(c: &mut Criterion) {
    let path = create_test_epub(10);
    
    c.bench_function("metadata_extraction", |b| {
        b.iter(|| {
            let mut handler = EpubHandler::new();
            handler.read_from_file(black_box(&path)).unwrap();
            handler.get_metadata().unwrap();
        });
    });
}

fn bench_image_optimization(c: &mut Criterion) {
    use ebook_cli::image_optimizer::OptimizationOptions;
    
    let path = create_test_cbz(5);
    
    c.bench_function("image_optimization", |b| {
        b.iter(|| {
            let mut handler = CbzHandler::new();
            handler.read_from_file(&path).unwrap();
            
            let options = OptimizationOptions::default();
            handler.optimize_images(black_box(options)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_epub_write,
    bench_epub_read,
    bench_cbz_write,
    bench_cbz_read,
    bench_metadata_extraction,
    bench_image_optimization
);
criterion_main!(benches);
