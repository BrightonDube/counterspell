// Test-only known-correct Java fixture for rotated-pantry.
public class Solution {
    public static long find_in_rotated(long[] stores, long target) {
        long lo = 0, hi = stores.length - 1;
        while (lo <= hi) {
            int mid = (int) ((lo + hi) / 2);
            if (stores[mid] == target) return mid;
            if (stores[(int) lo] <= stores[mid]) {
                if (stores[(int) lo] <= target && target < stores[mid]) { hi = mid - 1; } else { lo = mid + 1; }
            } else {
                if (stores[mid] < target && target <= stores[(int) hi]) { lo = mid + 1; } else { hi = mid - 1; }
            }
        }
        return -1;
    }
}
