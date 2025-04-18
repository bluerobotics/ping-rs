mod helper;

use bluerobotics_ping::{message::ProtocolMessage, Messages};
use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BatchSize, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};
use helper::protocol_message_from_messages;
use rand::{rngs::StdRng, SeedableRng};

fn benchmark_protocol_message_to_messages(c: &mut Criterion) {
    let seed = 42;
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("protocol_message_to_messages");
    group
        .measurement_time(std::time::Duration::from_secs(10))
        .significance_level(0.01)
        .sample_size(1000)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 100, 10000] {
        group.throughput(Throughput::Elements(*messages_count));

        let protocol_messages = helper::create_random_messages(&mut rng, *messages_count as usize)
            .iter()
            .map(protocol_message_from_messages)
            .collect::<Vec<ProtocolMessage>>();

        group.bench_function(BenchmarkId::new("sync", messages_count), |b| {
            b.to_async(&helper::rt()).iter_batched(
                || protocol_messages.clone(),
                |protocol_messages| async move {
                    for protocol_message in &protocol_messages {
                        let _message = black_box(Messages::try_from(protocol_message).unwrap());
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn benchmark_messages_to_protocol_message(c: &mut Criterion) {
    let seed = 42;
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("messages_to_protocol_message");
    group
        .measurement_time(std::time::Duration::from_secs(10))
        .significance_level(0.01)
        .sample_size(1000)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 100, 10000] {
        group.throughput(Throughput::Elements(*messages_count));

        let messages = helper::create_random_messages(&mut rng, *messages_count as usize);

        group.bench_function(BenchmarkId::new("sync", messages_count), |b| {
            b.to_async(&helper::rt()).iter_batched(
                || messages.clone(),
                |messages| async move {
                    for message in &messages {
                        let _protocol_message = black_box(protocol_message_from_messages(message));
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_protocol_message_to_messages,
    benchmark_messages_to_protocol_message
);
criterion_main!(benches);
