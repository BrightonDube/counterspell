module.exports = {
  token_window: (tokens, w, mark) => {
    const width = Math.min(w, tokens.length);
    if (width <= 0) return 0;
    let cur = 0;
    for (let i = 0; i < width; i++) if (tokens[i] === mark) cur++;
    let best = cur;
    for (let i = width; i < tokens.length; i++) {
      cur += (tokens[i] === mark ? 1 : 0) - (tokens[i - width] === mark ? 1 : 0);
      if (cur > best) best = cur;
    }
    return best;
  },
};
