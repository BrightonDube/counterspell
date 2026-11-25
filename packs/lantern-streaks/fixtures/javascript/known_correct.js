module.exports = {
  lit_streak: (lights) => {
    let streak = 0;
    for (let index = lights.length - 1; index >= 0; index -= 1) {
      if (lights[index] === 0) break;
      streak += 1;
    }
    return streak;
  },
};
