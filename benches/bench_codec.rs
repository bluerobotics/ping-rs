mod helper;

use bluerobotics_ping::{codec::PingCodec, message::ProtocolMessage};
use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BatchSize, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};
use futures::{SinkExt, StreamExt};
use helper::protocol_message_from_messages;
use rand::{rngs::StdRng, SeedableRng};
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

fn benchmark_decode(c: &mut Criterion) {
    let seed = 42;
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("decode");
    group
        .measurement_time(std::time::Duration::from_secs(10))
        .significance_level(0.01)
        .sample_size(1000)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 10, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(*messages_count));

        let buffer = helper::create_random_protocol_messages(&mut rng, *messages_count as usize)
            .iter()
            .map(|protocol_message| protocol_message.serialized())
            .flatten()
            .collect::<Vec<u8>>();

        group.bench_function(BenchmarkId::new("sync", messages_count), |b| {
            b.to_async(&helper::rt()).iter_batched(
                || {
                    let buffer = bytes::BytesMut::from(buffer.as_slice());
                    (buffer, PingCodec::new())
                },
                |(mut buffer, mut codec)| async move {
                    for _ in 0..*messages_count {
                        let _protocol_message =
                            black_box(codec.decode(&mut buffer).unwrap().unwrap());
                    }
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("async-framed", messages_count), |b| {
            b.to_async(&helper::rt()).iter_batched(
                || {
                    let codec = PingCodec::new();
                    let framed_read = FramedRead::new(buffer.as_slice(), codec);

                    framed_read
                },
                |mut framed_read| async move {
                    for _ in 0..*messages_count {
                        let _protocol_message =
                            black_box(framed_read.next().await.unwrap().unwrap());
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn benchmark_encode(c: &mut Criterion) {
    let seed = 42;
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("encode");
    group
        .measurement_time(std::time::Duration::from_secs(10))
        .significance_level(0.1)
        .sample_size(1000)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 10, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(*messages_count));

        let protocol_messages = helper::create_random_messages(&mut rng, *messages_count as usize)
            .iter()
            .map(protocol_message_from_messages)
            .collect::<Vec<ProtocolMessage>>();
        let buffer: Vec<u8> =
            Vec::with_capacity(*messages_count as usize * size_of::<ProtocolMessage>());

        group.bench_function(BenchmarkId::new("sync", messages_count), |b| {
            b.to_async(&helper::rt()).iter_batched(
                || {
                    let buffer = bytes::BytesMut::from(buffer.as_slice());
                    (protocol_messages.clone(), buffer, PingCodec::new())
                },
                |(protocol_messages, mut buffer, mut codec)| async move {
                    for protocol_message in protocol_messages {
                        codec.encode(protocol_message, &mut buffer).unwrap();
                    }
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("async-framed", messages_count), |b| {
            b.to_async(&helper::rt()).iter_batched(
                || {
                    let codec = PingCodec::new();
                    let framed_write = FramedWrite::new(buffer.clone(), codec);

                    (protocol_messages.clone(), framed_write)
                },
                |(messages, mut framed_write)| async move {
                    for protocol_message in messages {
                        framed_write.send(protocol_message).await.unwrap();
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_decode, benchmark_encode);
criterion_main!(benches);
