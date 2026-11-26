module.exports = {
  page_turn: (pages, f) => {
    const folio = [];
    let faults = 0;
    for (const p of pages) {
      if (!folio.includes(p)) {
        faults += 1;
        if (folio.length === f) folio.shift();
        folio.push(p);
      }
    }
    return faults;
  },
};
