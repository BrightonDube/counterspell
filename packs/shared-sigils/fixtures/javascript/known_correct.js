module.exports = {
  shared_marks: (first, second) => {
    const shared = [];
    let a = 0;
    let b = 0;
    while (a < first.length && b < second.length) {
      if (first[a] < second[b]) {
        a += 1;
      } else if (second[b] < first[a]) {
        b += 1;
      } else {
        const mark = first[a];
        if (shared.length === 0 || shared[shared.length - 1] !== mark) {
          shared.push(mark);
        }
        a += 1;
        b += 1;
      }
    }
    return shared;
  },
};
