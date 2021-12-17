// Ref https://github.com/abonander/multipart/blob/0.7.0/src/client/mod.rs#L193

use std::io::{BufWriter, Error as IoError, Write};

#[derive(Debug)]
pub struct MultipartFormDataWriter<W>
where
    W: Write,
{
    buf_writer: BufWriter<W>,
    pub boundary: String,
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

    pub fn write_text_field(
        &mut self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<(), IoError> {
        self.write_field(name, value.as_ref(), None, None, None)
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

        self.buf_writer.write_all(b"\r\n")?;

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
        writer.write_text_field("foo", "bar").unwrap();
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

    #[test]
    fn test_httpbin() {
        use std::{collections::HashMap, time::Duration};

        use isahc::{
            config::Configurable as _,
            http::{header::CONTENT_TYPE, Method},
            HttpClient, ReadResponseExt as _, Request,
        };
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Body {
            form: HashMap<String, String>,
            headers: HashMap<String, String>,
        }

        //
        let mut writer = MultipartFormDataWriter::new(vec![]);
        let boundary = writer.boundary.to_owned();
        writer.write_text_field("foo", "bar").unwrap();
        let buf = writer.finish().unwrap();

        //
        let request = Request::builder()
            .method(Method::POST)
            .uri("http://httpbin.org/post")
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(buf)
            .unwrap();

        let client = HttpClient::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let mut response = match client.send(request) {
            Ok(x) => x,
            Err(err) => {
                eprintln!("client.send failed, err: {}", err);
                return;
            }
        };

        let body = response.bytes().unwrap();
        let body: Body = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            body.form,
            vec![("foo".to_owned(), "bar".to_owned())]
                .into_iter()
                .collect(),
        );
        assert_eq!(
            body.headers.get("Content-Type").cloned().unwrap(),
            format!("multipart/form-data; boundary={}", boundary)
        );
        assert_eq!(body.headers.get("Content-Length").cloned().unwrap(), "141");
    }
}
