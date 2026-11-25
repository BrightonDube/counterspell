// Test-only known-correct Java fixture for lantern-streaks (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long lit_streak(long[] lights) {
        long streak = 0;
        for (int index = lights.length - 1; index >= 0; index -= 1) {
            if (lights[index] == 0) break;
            streak += 1;
        }
        return streak;
    }
}
