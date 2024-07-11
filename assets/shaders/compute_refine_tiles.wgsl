fn parent_index(id: u32) -> i32 {
    return i32(MAX_TILE_COUNT - 1u) * clamp(parameters.counter, 0, 1) - i32(id) * parameters.counter;
}
fn child_index() -> i32 {
    return atomicAdd(&parameters.child_index, parameters.counter);
}
fn final_index() -> i32 { return atomicAdd(&parameters.final_index, 1); }

@compute @workgroup_size(1, 1, 1)
fn prepare_next() {
    if parameters.counter == 1 {
        parameters.tile_count = u32(atomicExchange(&parameters.child_index, i32(MAX_TILE_COUNT - 1u)));
    } else {
        parameters.tile_count = MAX_TILE_COUNT - 1u - u32(atomicExchange(&parameters.child_index, 0));
    }
    indirect_buffer.workgroup_count.x = (parameters.tile_count + 63u) / 64u;
    parameters.counter = -parameters.counter;
}
