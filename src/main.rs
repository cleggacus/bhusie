fn main() {
    stacker::grow(1024 * 1024 * 1024, || {
        pollster::block_on(ray_tracer::app::run());
    });
}
