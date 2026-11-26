module.exports = {
  first_ancestor: (vals, left, right, a, b) => {
    const n = vals.length;
    if (n === 0 || a < 0 || a >= n || b < 0 || b >= n) return -1;
    const parent = new Map(), seen = new Set(), stack = [0];
    while (stack.length) {
      const i = stack.pop();
      if (seen.has(i)) continue;
      seen.add(i);
      for (const arr of [left, right]) {
        const k = arr[i] >= 0 && arr[i] < n ? arr[i] : -1;
        if (k !== -1 && !parent.has(k)) { parent.set(k, i); stack.push(k); }
      }
    }
    if (!seen.has(a) || !seen.has(b)) return -1;
    const anc = new Set();
    for (let x = a; x !== -1; x = parent.has(x) ? parent.get(x) : -1) anc.add(x);
    for (let x = b; x !== -1; x = parent.has(x) ? parent.get(x) : -1) {
      if (anc.has(x)) return x;
    }
    return -1;
  },
};
