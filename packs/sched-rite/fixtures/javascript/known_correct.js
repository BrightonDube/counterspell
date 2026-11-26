module.exports = {
  sched_rite: (pairs) => {
    const adj = new Map();
    for (const [a, b] of pairs) {
      if (!adj.has(a)) adj.set(a, []);
      adj.get(a).push(b);
    }
    const state = new Map();
    const visit = (u) => {
      state.set(u, 1);
      for (const v of adj.get(u) || []) {
        const s = state.get(v) || 0;
        if (s === 1) return true;
        if (s === 0 && visit(v)) return true;
      }
      state.set(u, 2);
      return false;
    };
    for (const u of adj.keys()) {
      if ((state.get(u) || 0) === 0 && visit(u)) return 0;
    }
    return 1;
  },
};
