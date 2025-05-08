// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! APIs to help access bundles

use crate::index::SupportBundleIndex;
use anyhow::Result;
use async_trait::async_trait;
use camino::Utf8Path;
use std::io;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tokio::io::AsyncRead;
use tokio::io::ReadBuf;

/// An I/O source which can read to a buffer
///
/// This describes access to individual files within the bundle.
pub trait FileAccessor: AsyncRead + Unpin + Send {}
impl<T: AsyncRead + Unpin + Send + ?Sized> FileAccessor for T {}

pub type BoxedFileAccessor<'a> = Box<dyn FileAccessor + 'a>;

/// Describes how the support bundle's data and metadata are accessed.
#[async_trait]
pub trait SupportBundleAccessor: Send {
    /// Access the index of a support bundle
    async fn get_index(&self) -> Result<SupportBundleIndex>;

    /// Access a file within the support bundle
    async fn get_file<'a>(&mut self, path: &Utf8Path) -> Result<BoxedFileAccessor<'a>>
    where
        Self: 'a;
}

pub struct LocalFileAccess {
    archive: zip::read::ZipArchive<std::fs::File>,
}

impl LocalFileAccess {
    pub fn new(path: &Utf8Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        Ok(Self {
            archive: zip::read::ZipArchive::new(file)?,
        })
    }
}

// Access for: Local zip files
#[async_trait]
impl SupportBundleAccessor for LocalFileAccess {
    async fn get_index(&self) -> Result<SupportBundleIndex> {
        let names: Vec<&str> = self.archive.file_names().collect();
        let all_names = names.join("\n");
        Ok(SupportBundleIndex::new(&all_names))
    }

    async fn get_file<'a>(&mut self, path: &Utf8Path) -> Result<BoxedFileAccessor<'a>> {
        let mut file = self.archive.by_name(path.as_str())?;
        let mut buf = Vec::new();
        std::io::copy(&mut file, &mut buf)?;

        Ok(Box::new(AsyncZipFile { buf, copied: 0 }))
    }
}

// We're currently buffering the entire file into memory, mostly because dealing with the lifetime
// of ZipArchive and ZipFile objects is so difficult.
pub struct AsyncZipFile {
    buf: Vec<u8>,
    copied: usize,
}

impl AsyncRead for AsyncZipFile {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let to_copy = std::cmp::min(self.buf.len() - self.copied, buf.remaining());
        if to_copy == 0 {
            return Poll::Ready(Ok(()));
        }
        let src = &self.buf[self.copied..];
        buf.put_slice(&src[..to_copy]);
        self.copied += to_copy;
        Poll::Ready(Ok(()))
    }
}
