use crate::{Metadata, Result, EbookError};
use quick_xml::events::{Event, BytesEnd, BytesStart, BytesText};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

#[derive(Debug, Clone, Default)]
pub struct ComicInfo {
    pub title: Option<String>,
    pub series: Option<String>,
    pub number: Option<String>,
    pub volume: Option<String>,
    pub summary: Option<String>,
    pub publisher: Option<String>,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub inker: Option<String>,
    pub colorist: Option<String>,
    pub letterer: Option<String>,
    pub cover_artist: Option<String>,
    pub editor: Option<String>,
    pub year: Option<String>,
    pub month: Option<String>,
    pub day: Option<String>,
    pub language_iso: Option<String>,
    pub page_count: Option<u32>,
    pub genre: Option<String>,
    pub tags: Vec<String>,
    pub web: Option<String>,
}

impl ComicInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_metadata(metadata: &Metadata) -> Self {
        let mut comic_info = Self::new();
        
        comic_info.title = metadata.title.clone();
        comic_info.publisher = metadata.publisher.clone();
        comic_info.summary = metadata.description.clone();
        comic_info.language_iso = metadata.language.clone();
        
        if let Some(author) = &metadata.author {
            comic_info.writer = Some(author.clone());
        }
        
        if let Some(tags) = &metadata.tags {
            comic_info.tags = tags.clone();
        }
        
        comic_info
    }

    pub fn to_metadata(&self) -> Metadata {
        let mut metadata = Metadata::new();
        
        metadata.title = self.title.clone();
        metadata.publisher = self.publisher.clone();
        metadata.description = self.summary.clone();
        metadata.language = self.language_iso.clone();
        metadata.author = self.writer.clone();
        metadata.format = Some("CBZ".to_string());
        
        if !self.tags.is_empty() {
            metadata.tags = Some(self.tags.clone());
        }
        
        metadata
    }

    pub fn parse_xml(xml_content: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text(true);
        
        let mut comic_info = Self::new();
        let mut buf = Vec::new();
        let mut current_tag = String::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    match current_tag.as_str() {
                        "Title" => comic_info.title = Some(text),
                        "Series" => comic_info.series = Some(text),
                        "Number" => comic_info.number = Some(text),
                        "Volume" => comic_info.volume = Some(text),
                        "Summary" => comic_info.summary = Some(text),
                        "Publisher" => comic_info.publisher = Some(text),
                        "Writer" => comic_info.writer = Some(text),
                        "Penciller" => comic_info.penciller = Some(text),
                        "Inker" => comic_info.inker = Some(text),
                        "Colorist" => comic_info.colorist = Some(text),
                        "Letterer" => comic_info.letterer = Some(text),
                        "CoverArtist" => comic_info.cover_artist = Some(text),
                        "Editor" => comic_info.editor = Some(text),
                        "Year" => comic_info.year = Some(text),
                        "Month" => comic_info.month = Some(text),
                        "Day" => comic_info.day = Some(text),
                        "LanguageISO" => comic_info.language_iso = Some(text),
                        "PageCount" => {
                            if let Ok(count) = text.parse::<u32>() {
                                comic_info.page_count = Some(count);
                            }
                        }
                        "Genre" => comic_info.genre = Some(text),
                        "Tags" => {
                            for tag in text.split(',') {
                                let trimmed = tag.trim();
                                if !trimmed.is_empty() {
                                    comic_info.tags.push(trimmed.to_string());
                                }
                            }
                        }
                        "Web" => comic_info.web = Some(text),
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(EbookError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(comic_info)
    }

    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("utf-8"), None)))?;
        
        let mut comic_info_elem = BytesStart::new("ComicInfo");
        comic_info_elem.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
        comic_info_elem.push_attribute(("xmlns:xsd", "http://www.w3.org/2001/XMLSchema"));
        writer.write_event(Event::Start(comic_info_elem))?;
        
        self.write_element(&mut writer, "Title", &self.title)?;
        self.write_element(&mut writer, "Series", &self.series)?;
        self.write_element(&mut writer, "Number", &self.number)?;
        self.write_element(&mut writer, "Volume", &self.volume)?;
        self.write_element(&mut writer, "Summary", &self.summary)?;
        self.write_element(&mut writer, "Publisher", &self.publisher)?;
        self.write_element(&mut writer, "Writer", &self.writer)?;
        self.write_element(&mut writer, "Penciller", &self.penciller)?;
        self.write_element(&mut writer, "Inker", &self.inker)?;
        self.write_element(&mut writer, "Colorist", &self.colorist)?;
        self.write_element(&mut writer, "Letterer", &self.letterer)?;
        self.write_element(&mut writer, "CoverArtist", &self.cover_artist)?;
        self.write_element(&mut writer, "Editor", &self.editor)?;
        self.write_element(&mut writer, "Year", &self.year)?;
        self.write_element(&mut writer, "Month", &self.month)?;
        self.write_element(&mut writer, "Day", &self.day)?;
        self.write_element(&mut writer, "LanguageISO", &self.language_iso)?;
        
        if let Some(page_count) = self.page_count {
            self.write_element(&mut writer, "PageCount", &Some(page_count.to_string()))?;
        }
        
        self.write_element(&mut writer, "Genre", &self.genre)?;
        
        if !self.tags.is_empty() {
            let tags_str = self.tags.join(", ");
            self.write_element(&mut writer, "Tags", &Some(tags_str))?;
        }
        
        self.write_element(&mut writer, "Web", &self.web)?;
        
        writer.write_event(Event::End(BytesEnd::new("ComicInfo")))?;
        
        let result = writer.into_inner().into_inner();
        String::from_utf8(result)
            .map_err(|e| EbookError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }
    
    fn write_element<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        tag: &str,
        value: &Option<String>,
    ) -> Result<()> {
        if let Some(val) = value {
            writer.write_event(Event::Start(BytesStart::new(tag)))?;
            writer.write_event(Event::Text(BytesText::new(val)))?;
            writer.write_event(Event::End(BytesEnd::new(tag)))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comic_info_parse_and_generate() {
        let xml = r#"<?xml version="1.0"?>
<ComicInfo>
    <Title>Test Comic</Title>
    <Series>Test Series</Series>
    <Number>1</Number>
    <Writer>Test Writer</Writer>
    <Publisher>Test Publisher</Publisher>
    <Summary>A test comic book</Summary>
    <LanguageISO>en</LanguageISO>
    <PageCount>24</PageCount>
    <Tags>Action, Adventure</Tags>
</ComicInfo>"#;

        let comic_info = ComicInfo::parse_xml(xml).unwrap();
        assert_eq!(comic_info.title, Some("Test Comic".to_string()));
        assert_eq!(comic_info.series, Some("Test Series".to_string()));
        assert_eq!(comic_info.writer, Some("Test Writer".to_string()));
        assert_eq!(comic_info.page_count, Some(24));
        assert_eq!(comic_info.tags.len(), 2);
    }

    #[test]
    fn test_comic_info_to_xml() {
        let mut comic_info = ComicInfo::new();
        comic_info.title = Some("Test Comic".to_string());
        comic_info.writer = Some("Test Writer".to_string());
        comic_info.page_count = Some(24);
        
        let xml = comic_info.to_xml().unwrap();
        assert!(xml.contains("<Title>Test Comic</Title>"));
        assert!(xml.contains("<Writer>Test Writer</Writer>"));
        assert!(xml.contains("<PageCount>24</PageCount>"));
    }

    #[test]
    fn test_metadata_conversion() {
        let mut metadata = Metadata::new();
        metadata.title = Some("Comic Title".to_string());
        metadata.author = Some("Comic Author".to_string());
        metadata.publisher = Some("Comic Publisher".to_string());
        
        let comic_info = ComicInfo::from_metadata(&metadata);
        assert_eq!(comic_info.title, Some("Comic Title".to_string()));
        assert_eq!(comic_info.writer, Some("Comic Author".to_string()));
        
        let converted_metadata = comic_info.to_metadata();
        assert_eq!(converted_metadata.title, Some("Comic Title".to_string()));
        assert_eq!(converted_metadata.author, Some("Comic Author".to_string()));
    }
}
