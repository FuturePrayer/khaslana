//! 文件级调用暂存与持久化。调用点不进入节点属性，也不复制文件路径和导入表。
//! BLOB 中的字符串字典仅在单个文件内构建，避免落盘时再次生成整图 JSON。

use crate::types::domain_ui_resources;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::Result;

#[derive(Debug)]
pub(super) struct StoredCall {
    pub source_qn: Arc<str>,
    pub callee_display: Arc<str>,
    pub name: Arc<str>,
    pub scope: Arc<str>,
    pub line: u32,
    pub java: Arc<str>,
}

#[derive(Debug, Default)]
pub(super) struct FileCalls {
    pub imports: Vec<String>,
    pub calls: Vec<StoredCall>,
}

impl FileCalls {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut dictionary = HashMap::<&str, u32>::new();
        let mut strings = Vec::new();
        fn intern<'a>(dictionary: &mut HashMap<&'a str, u32>, strings: &mut Vec<&'a str>, value: &'a str) -> Result<u32> {
            if let Some(id) = dictionary.get(value) { return Ok(*id); }
            let id = u32::try_from(strings.len()).map_err(|_| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::TOO_MANY_STRINGS_IN_CALL_RECORDS, &[])))?;
            dictionary.insert(value, id);
            strings.push(value);
            Ok(id)
        }
        let imports = self.imports.iter().map(|value| intern(&mut dictionary, &mut strings, value)).collect::<Result<Vec<_>>>()?;
        let calls = self.calls.iter().map(|call| Ok([
            intern(&mut dictionary, &mut strings, &call.source_qn)?, intern(&mut dictionary, &mut strings, &call.callee_display)?, intern(&mut dictionary, &mut strings, &call.name)?, intern(&mut dictionary, &mut strings, &call.scope)?,
            call.line,
            intern(&mut dictionary, &mut strings, &call.java)?,
        ])).collect::<Result<Vec<_>>>()?;
        let mut bytes = b"KCALL3".to_vec();
        let put = |bytes: &mut Vec<u8>, value: usize| -> Result<()> {
            bytes.extend_from_slice(&u32::try_from(value).map_err(|_| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CALL_RECORDS_ARE_TOO_LARGE, &[])))?.to_le_bytes());
            Ok(())
        };
        put(&mut bytes, strings.len())?;
        for string in strings { put(&mut bytes, string.len())?; bytes.extend_from_slice(string.as_bytes()); }
        put(&mut bytes, imports.len())?;
        for id in imports { bytes.extend_from_slice(&id.to_le_bytes()); }
        put(&mut bytes, calls.len())?;
        for call in calls { for id in call { bytes.extend_from_slice(&id.to_le_bytes()); } }
        Ok(bytes)
    }

    pub fn decode(mut bytes: &[u8]) -> Result<Self> {
        let with_java = bytes.starts_with(b"KCALL3");
        let with_lines = with_java || bytes.starts_with(b"KCALL2");
        if !with_lines && !bytes.starts_with(b"KCALL1") { return Err(crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::INCOMPATIBLE_CALL_RECORD_FORMAT, &[]))); }
        bytes = &bytes[6..];
        fn number(bytes: &mut &[u8]) -> Result<usize> {
            let raw = bytes.get(..4).ok_or_else(|| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::TRUNCATED_CALL_RECORDS, &[])))?;
            let value = u32::from_le_bytes(raw.try_into().unwrap()) as usize;
            *bytes = &bytes[4..];
            Ok(value)
        }
        let count = number(&mut bytes)?;
        if count > bytes.len() / 4 { return Err(crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CORRUPT_CALL_RECORD_DICTIONARY, &[]))); }
        let mut strings = Vec::<Arc<str>>::with_capacity(count);
        for _ in 0..count {
            let len = number(&mut bytes)?;
            let raw = bytes.get(..len).ok_or_else(|| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::TRUNCATED_CALL_RECORD_STRING, &[])))?;
            strings.push(std::str::from_utf8(raw).map_err(|_| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CORRUPT_CALL_RECORD_ENCODING, &[])))?.into());
            bytes = &bytes[len..];
        }
        let string = |bytes: &mut &[u8]| -> Result<Arc<str>> {
            strings.get(number(bytes)?).cloned().ok_or_else(|| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CORRUPT_CALL_RECORD_DICTIONARY_REFERENCE, &[])))
        };
        let count = number(&mut bytes)?;
        if count > bytes.len() / 4 { return Err(crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CORRUPT_CALL_RECORD_IMPORT_TABLE, &[]))); }
        let mut imports = Vec::with_capacity(count);
        for _ in 0..count { imports.push(string(&mut bytes)?.to_string()); }
        let count = number(&mut bytes)?;
        if count > bytes.len() / if with_java { 24 } else if with_lines { 20 } else { 16 } { return Err(crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CORRUPT_CALL_RECORD_LIST, &[]))); }
        let mut calls = Vec::with_capacity(count);
        for _ in 0..count {
            calls.push(StoredCall { source_qn: string(&mut bytes)?, callee_display: string(&mut bytes)?,
                name: string(&mut bytes)?, scope: string(&mut bytes)?, line: if with_lines { number(&mut bytes)? as u32 } else { 0 },
                java: if with_java { string(&mut bytes)? } else { "".into() } });
        }
        if !bytes.is_empty() { return Err(crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CALL_RECORDS_CONTAIN_TRAILING_DATA, &[]))); }
        Ok(Self { imports, calls })
    }
}
