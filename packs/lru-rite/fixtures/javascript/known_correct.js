module.exports = {
  lru_rite: (pages, f) => {
    const folio = [];
    let faults = 0;
    for (const p of pages) {
      const at = folio.indexOf(p);
      if (at !== -1) {
        folio.splice(at, 1);
        folio.push(p);
      } else {
        faults += 1;
        if (folio.length === f) folio.shift();
        folio.push(p);
      }
    }
    return faults;
  },
};
