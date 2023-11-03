
use std::io::Read;

use claxon::{metadata::{StreamInfo, VorbisComment}, input::ReadBytes};
#[derive(Clone,Debug)]
pub struct Picture{
    pub picture_type:u32,
    pub mime:String,
    pub description:String,
    pub width:u32,
    pub height:u32,
    pub depth:u32,
    pub index_colors:u32,
    pub image:Vec<u8>,
}
pub struct ParsedMetadata{
    pub info:Option<StreamInfo>,
    pub vorbis_comment: Option<VorbisComment>,
    pub picture:Option<Picture>,
}
pub fn load_flac<R:Read>(reader:R)->claxon::Result<(claxon::FlacReader<R>,ParsedMetadata)>{
    let mut meta=ParsedMetadata{
        info:None,
        vorbis_comment:None,
        picture:None,
    };
    let mut buf_reader = claxon::input::BufferedReader::new(reader);
    claxon::read_stream_header(&mut buf_reader)?;
    loop{
        let metadata=claxon::metadata::read_metadata_block_header(&mut buf_reader)?;
        read_metadata_block(&mut buf_reader,metadata.block_type,metadata.length,&mut meta)?;
        if metadata.is_last{
            break;
        }
    }
    match meta.info{
        Some(streaminfo)=>{
            Ok((claxon::FlacReader::from_metadata(buf_reader, streaminfo),meta))
        },
        None=>{
            Err(claxon::Error::FormatError("No Streaminfo"))
        }
    }
}
//https://xiph.org/flac/format.html#metadata_block_picture
#[inline]
pub fn read_metadata_block<R: ReadBytes>(input: &mut R,
                                        block_type: u8,
                                        length: u32,
                                        metadata:&mut ParsedMetadata,
                                    )-> claxon::Result<()> {
    match block_type {
        0 => {
            // The streaminfo block has a fixed size of 34 bytes.
            if length == 34 {
                let streaminfo = claxon::metadata::read_streaminfo_block(input)?;
                metadata.info.replace(streaminfo);
                Ok(())
            } else {
                Err(claxon::Error::FormatError("invalid streaminfo metadata block length"))
            }
        }
        1 => {
            claxon::metadata::read_padding_block(input, length)?;
            Ok(())
        }
        4 => {
            let vorbis_comment = claxon::metadata::read_vorbis_comment_block(input, length)?;
            metadata.vorbis_comment.replace(vorbis_comment);
            Ok(())
        }
        6 => {
            let picture_type=input.read_be_u32()?;
            let len=input.read_be_u32()?;
            let mut mime=vec![0;len as usize];
            input.read_into(&mut mime).map_err(|e|claxon::Error::IoError(e))?;
            let len=input.read_be_u32()?;
            let mut description=vec![0;len as usize];
            input.read_into(&mut description).map_err(|e|claxon::Error::IoError(e))?;
            let width=input.read_be_u32()?;
            let height=input.read_be_u32()?;
            let depth=input.read_be_u32()?;
            let index_colors=input.read_be_u32()?;
            let len=input.read_be_u32()?;
            let mut image=vec![0;len as usize];
            input.read_into(&mut image).map_err(|e|claxon::Error::IoError(e))?;
            if let Some(mime)=String::from_utf8(mime).ok(){
                let description=match String::from_utf8(description).ok(){
                    Some(s)=>s,
                    None=>String::new(),
                };
                metadata.picture.replace(Picture{
                    picture_type,
                    mime,
                    description,
                    width,
                    height,
                    depth,
                    index_colors,
                    image,
                });
            }
            Ok(())
        }
        127 => {
            Err(claxon::Error::FormatError("invalid streaminfo metadata block length"))
        }
        _ => {
            input.skip(length)?;
            Ok(())
        }
    }
}
fn main(){
    let mut no_args = true;
    for fname in std::env::args().skip(1) {
        no_args = false;

        print!("{}", fname);
        let reader = std::fs::File::open(&fname).unwrap();
        let (flac,exmeta)=load_flac(reader).unwrap();
        println!("{:?}",exmeta.picture);
        println!("{:?}",exmeta.info);
        println!("{:?}",flac.streaminfo());
        println!(": done");
    }

    if no_args {
        println!("no files to decode");
    }
}
