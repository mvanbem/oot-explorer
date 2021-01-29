export function clamp(x: number, min: number, max: number): number {
    if (x < min) {
        return min;
    } else if (x <= max) {
        return x;
    } else {
        return max;
    }
}
