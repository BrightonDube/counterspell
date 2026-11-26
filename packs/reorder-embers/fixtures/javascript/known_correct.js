module.exports = {
  reorder_embers: (marks, nxt) => {
    const order = [];
    for (let i = nxt.length ? 0 : -1; i !== -1; i = nxt[i]) { order.push(marks[i]); }
    const half = Math.ceil(order.length / 2);
    const front = order.slice(0, half), back = order.slice(half).reverse();
    const out = [];
    for (let k = 0; k < front.length; k += 1) {
      out.push(front[k]);
      if (k < back.length) { out.push(back[k]); }
    }
    return out;
  },
};
