use std::fs::File;
use ac_ffmpeg::{
    format::{io::IO, demuxer::Demuxer},
    codec::{audio::{AudioResampler, AudioDecoder, AudioEncoder}, Decoder},
};

fn main() -> anyhow::Result<()> {
    let mut wav_file = File::open("tada-81529.mp3")?;
    let io = IO::from_seekable_read_stream(&mut wav_file);
    let mut demuxer = Demuxer::builder()
        .build(io)?
        .find_stream_info(None)
        .map_err(|(_, e)| anyhow::anyhow!("Failed to find stream info: {}", e))?;

    let (index, binding) = demuxer
        .streams()
        .iter()
        .map(|stream| stream.codec_parameters())
        .enumerate()
        .find(|(_, params)| params.is_audio_codec())
        .ok_or_else(|| anyhow::anyhow!("No audio stream found"))?;
    let codec_params = binding.as_audio_codec_parameters()
        .unwrap();
    
    let mut decoder = AudioDecoder::from_codec_parameters(&codec_params)?
        .build()?;
    
    let mut resampler = AudioResampler::builder()
        .source_channel_layout(codec_params.channel_layout().to_owned())
        .source_sample_format(codec_params.sample_format())
        .source_sample_rate(codec_params.sample_rate())
        .target_channel_layout(codec_params.channel_layout().to_owned())
        .target_sample_format(codec_params.sample_format())
        .target_sample_rate(48000)
        .build()?;
    let mut output = File::create("output.wav")?;
    let mut io_output = IO::from_seekable_read_stream(&mut output);
    
    while let Some(packet) = demuxer.take()? {
        if packet.stream_index() == index {
            decoder.push(packet)?;

            while let Some(frame) = decoder.take()? {
                resampler.push(frame)?;

                while let Some(frame) = resampler.take()? {

                }
            }
        }
    }
    println!("Finished");
    resampler.flush()?;
    Ok(())
}
