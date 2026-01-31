use ebook_cli::formats::TxtHandler;
use ebook_cli::traits::{EbookReader, EbookWriter};

#[test]
fn test_trait_read_from_bytes_txt() {
    let mut h = TxtHandler::new();
    let data = b"Hello\nWorld\n";
    h.read_from_bytes(data).unwrap();
    let content = h.get_content().unwrap();
    assert_eq!(content, "Hello\nWorld\n");
}

#[test]
fn test_trait_read_from_reader_txt() {
    let mut h = TxtHandler::new();
    let data = b"Line1\nLine2\n";
    let cursor = std::io::Cursor::new(data);
    h.read_from_reader(cursor).unwrap();
    let content = h.get_content().unwrap();
    assert_eq!(content, "Line1\nLine2\n");
}

#[test]
fn test_trait_write_to_writer_txt() {
    let mut h = TxtHandler::new();
    h.set_content("abc\n123\n").unwrap();

    let mut out = Vec::<u8>::new();
    {
        let cursor = std::io::Cursor::new(&mut out);
        h.write_to_writer(cursor).unwrap();
    }

    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("abc"));
    assert!(s.contains("123"));
}

#[test]
fn test_trait_streaming_helpers_are_concurrency_safe() {
    let threads = 16usize;

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(threads);

        for i in 0..threads {
            handles.push(scope.spawn(move || {
                let mut r = TxtHandler::new();
                let data = format!("thread-{i}\n").into_bytes();
                r.read_from_bytes(&data).unwrap();
                let content = r.get_content().unwrap();
                assert_eq!(content, format!("thread-{i}\n"));

                let mut w = TxtHandler::new();
                w.set_content(&format!("payload-{i}\n")).unwrap();
                let mut out = Vec::<u8>::new();
                {
                    let cursor = std::io::Cursor::new(&mut out);
                    w.write_to_writer(cursor).unwrap();
                }
                let s = String::from_utf8(out).unwrap();
                assert!(s.contains(&format!("payload-{i}")));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    });
}
