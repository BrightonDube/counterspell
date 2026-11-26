module.exports = {
  alloc_rite: (blocks, reqs) => {
    const slots = [...blocks];
    let granted = 0;
    for (const r of reqs) {
      for (let i = 0; i < slots.length; i++) {
        if (slots[i] >= r) { slots[i] -= r; granted += 1; break; }
      }
    }
    return granted;
  },
};
