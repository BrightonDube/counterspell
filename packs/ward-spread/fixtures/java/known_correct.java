// Test-only known-correct Java fixture for ward-spread.
public class Solution {
    public static long spread_ways(long cells, long guards) {
        if (guards == 0) {
            return 1;
        }
        if (cells <= 0) {
            return 0;
        }
        return spread_ways(cells - 2, guards - 1) + spread_ways(cells - 1, guards);
    }
}
