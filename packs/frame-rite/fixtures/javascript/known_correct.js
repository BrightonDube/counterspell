module.exports = {
  frame_rite: (pages, f) => {
    const frame = [];
    let faults = 0;
    const n = pages.length;
    for (let i = 0; i < n; i++) {
      const p = pages[i];
      if (frame.includes(p)) continue;
      faults += 1;
      if (frame.length < f) { frame.push(p); continue; }
      let worst = 0, worstNext = -1;
      for (let k = 0; k < frame.length; k++) {
        let nxt = n + 1;
        for (let j = i + 1; j < n; j++) if (pages[j] === frame[k]) { nxt = j; break; }
        if (nxt > worstNext) { worst = k; worstNext = nxt; }
      }
      frame[worst] = p;
    }
    return faults;
  },
};
