// Ref https://github.com/abonander/multipart/blob/0.7.0/src/client/mod.rs#L193

use std::io::{BufWriter, Error as IoError, Write};

#[derive(Debug)]
pub struct MultipartFormDataWriter<W>
where
    W: Write,
{
    buf_writer: BufWriter<W>,
    boundary: String,
}

impl<W> MultipartFormDataWriter<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            buf_writer: BufWriter::new(writer),
            boundary: multipart_boundary::generate(),
        }
    }

    pub fn with_boundary(writer: W, boundary: impl AsRef<str>) -> Self {
        Self {
            buf_writer: BufWriter::new(writer),
            boundary: boundary.as_ref().to_owned(),
        }
    }

    pub fn write_field<'a>(
        &mut self,
        name: impl AsRef<str>,
        value: impl AsRef<[u8]>,
        filename: impl Into<Option<&'a str>>,
        content_type: impl Into<Option<&'a str>>,
        headers: impl Into<Option<Vec<(&'a str, &'a str)>>>,
    ) -> Result<(), IoError> {
        self.buf_writer.write_all(b"--")?;
        self.buf_writer.write_all(self.boundary.as_bytes())?;
        self.buf_writer.write_all(b"\r\n")?;

        self.buf_writer
            .write_all(br#"Content-Disposition: form-data; name=""#)?;
        self.buf_writer.write_all(name.as_ref().as_bytes())?;
        self.buf_writer.write_all(br#"""#)?;

        if let Some(filename) = filename.into() {
            self.buf_writer.write_all(br#"; filename=""#)?;
            self.buf_writer.write_all(filename.as_bytes())?;
            self.buf_writer.write_all(br#"""#)?;
        }

        self.buf_writer.write_all(b"\r\n")?;

        if let Some(content_type) = content_type.into() {
            self.buf_writer.write_all(b"Content-Type: ")?;
            self.buf_writer.write_all(content_type.as_bytes())?;
            self.buf_writer.write_all(b"\r\n")?;
        }
        if let Some(headers) = headers.into() {
            for (k, v) in headers.into_iter() {
                self.buf_writer.write_all(k.as_bytes())?;
                self.buf_writer.write_all(b": ")?;
                self.buf_writer.write_all(v.as_bytes())?;
                self.buf_writer.write_all(b"\r\n")?;
            }
        }

        self.buf_writer.write_all(b"\r\n")?;

        self.buf_writer.write_all(value.as_ref())?;
        self.buf_writer.write_all(b"\r\n")?;

        Ok(())
    }

    pub fn finish(mut self) -> Result<W, IoError> {
        self.buf_writer.write_all(b"--")?;
        self.buf_writer.write_all(self.boundary.as_bytes())?;
        self.buf_writer.write_all(b"--")?;

        Ok(self.buf_writer.into_inner()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer() {
        //
        let mut writer = MultipartFormDataWriter::with_boundary(
            vec![],
            "------------------------afb08437765cfecd",
        );
        writer.write_field("foo", "bar", None, None, None).unwrap();
        let buf = writer.finish().unwrap();
        println!("{}", String::from_utf8(buf.clone()).unwrap());
        assert_eq!(buf, include_bytes!("../tests/curl_F_body_files/case1.txt"));

        //
        let mut writer = MultipartFormDataWriter::with_boundary(
            vec![],
            "------------------------8cde15cb2484c740",
        );
        writer
            .write_field(
                "foo",
                "bar",
                "foo.txt",
                "text/plain",
                vec![("X-A", "1"), ("X-B", "2")],
            )
            .unwrap();
        let buf = writer.finish().unwrap();
        println!("{}", String::from_utf8(buf.clone()).unwrap());
        assert_eq!(buf, include_bytes!("../tests/curl_F_body_files/case2.txt"));
    }
}
