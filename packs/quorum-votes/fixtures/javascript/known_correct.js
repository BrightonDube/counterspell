module.exports = {
  quorum_reached: (n, votes) => (votes >= Math.floor(n / 2) + 1 ? 1 : 0),
};
