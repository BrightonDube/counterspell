// Test-only known-correct Java fixture for summon-rite.
public class Solution {
    static boolean go(long[] marks, int i, long need) {
        if (need == 0) {
            return true;
        }
        if (i >= marks.length) {
            return false;
        }
        return go(marks, i + 1, need - marks[i]) || go(marks, i + 1, need);
    }
    public static long can_summon(long[] marks, long target) {
        return go(marks, 0, target) ? 1 : 0;
    }
}
