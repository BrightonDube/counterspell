// Test-only known-correct Java fixture for twin-tides (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[]).
public class Solution {
    public static long[] merge_tides(long[] first, long[] second) {
        long[] merged = new long[first.length + second.length];
        int a = 0, b = 0, write = 0;
        while (a < first.length && b < second.length) {
            merged[write++] = first[a] <= second[b] ? first[a++] : second[b++];
        }
        while (a < first.length) merged[write++] = first[a++];
        while (b < second.length) merged[write++] = second[b++];
        return merged;
    }
}
