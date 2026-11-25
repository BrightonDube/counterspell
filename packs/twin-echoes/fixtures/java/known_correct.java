// Test-only known-correct Java fixture for twin-echoes (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long repeating_echoes(long[] echoes) {
        long[] counts = new long[51];
        for (long echo : echoes) counts[(int) echo] += 1;
        long total = 0;
        for (long count : counts) {
            if (count >= 2) total += 1;
        }
        return total;
    }
}
