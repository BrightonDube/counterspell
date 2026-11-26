// Test-only known-correct Java fixture for leak-check.
public class Solution {
    public static long leak_check(long[] train, long[] test) {
        java.util.Set<Long> tr = new java.util.HashSet<>();
        for (long x : train) tr.add(x);
        java.util.Set<Long> both = new java.util.HashSet<>();
        for (long x : test) if (tr.contains(x)) both.add(x);
        return both.size();
    }
}
