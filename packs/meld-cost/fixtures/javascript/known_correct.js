module.exports = {
  meld_cost: (sizes) => {
    const heap = [...sizes].sort((a, b) => a - b);
    let cost = 0;
    while (heap.length > 1) {
      const fused = heap.shift() + heap.shift();
      cost += fused;
      let i = 0;
      while (i < heap.length && heap[i] < fused) i += 1;
      heap.splice(i, 0, fused);
    }
    return cost;
  },
};
