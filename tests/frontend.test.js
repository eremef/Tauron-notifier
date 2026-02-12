/** @vitest-environment jsdom */
import { describe, it, expect } from 'vitest';
const { filterOutages, formatDate } = require('../public/script.js');

describe('Frontend Logic', () => {
    describe('filterOutages', () => {
        const mockOutages = [
            { Message: 'Planned outage at Rozbrat 12, Wrocław' },
            { Message: 'Maintenance on Legnicka 5, Wrocław' }
        ];

        it('finds outages matching the street name', () => {
            const filtered = filterOutages(mockOutages, 'Rozbrat');
            expect(filtered).toHaveLength(1);
            expect(filtered[0].Message).toContain('Rozbrat');
        });

        it('returns empty array when no match found', () => {
            const filtered = filterOutages(mockOutages, 'Main Street');
            expect(filtered).toHaveLength(0);
        });

        it('returns empty array when street name is empty', () => {
            const filtered = filterOutages(mockOutages, '');
            expect(filtered).toHaveLength(0);
        });
    });

    describe('formatDate', () => {
        it('formats a date string correctly in pl-PL locale', () => {
            // "2024-02-12T10:30:00"
            const dateStr = '2024-02-12T10:30:00';
            const formatted = formatDate(dateStr);
            // Expected format depends on exact system locale, but should contain these components
            // In pl-PL: "pn., 12 lut, 10:30" or similar
            expect(formatted).toMatch(/12/);
            expect(formatted).toMatch(/10:30/);
        });

        it('returns empty string for null input', () => {
            expect(formatDate(null)).toBe('');
            expect(formatDate('')).toBe('');
        });
    });
});
