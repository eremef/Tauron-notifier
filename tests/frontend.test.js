/** @vitest-environment jsdom */
import { describe, it, expect } from 'vitest';
const { filterOutages, formatDate } = require('../public/script.js');

describe('Frontend Logic', () => {
    describe('filterOutages', () => {
        const mockOutages = [
            { Message: 'Planned outage at Henryka Probusa 12, Wrocław', GAID: 100 },
            { Message: 'Awaria na Probusa 5', GAID: 101 },
            { Message: 'Maintenance on Legnicka 5, Wrocław', GAID: 102 },
            { Message: 'Prace na Jana Pawła II', GAID: 103 },
            { Message: 'Utrudnienia na Pawła', GAID: 104 },
            { Message: 'Wrocław Probusa..', GAID: 105 }
        ];

        it('finds outages matching the full street name', () => {
            const filtered = filterOutages(mockOutages, 'Henryka Probusa', { streetGAID: 0 });
            expect(filtered.some(o => o.Message.includes('Henryka Probusa'))).toBe(true);
        });

        it('finds outages matching the short street name (last part)', () => {
            const filtered = filterOutages(mockOutages, 'Henryka Probusa', { streetGAID: 0 });
            expect(filtered.some(o => o.Message.includes('Awaria na Probusa'))).toBe(true);
        });

        it('finds outages matching significant parts (ignoring short words)', () => {
            const filtered = filterOutages(mockOutages, 'Jana Pawła II', { streetGAID: 0 });
            expect(filtered.some(o => o.Message.includes('Pawła'))).toBe(true);
        });

        it('finds outages by GAID even if text does not match', () => {
            // "Rozbrat" search should find "Wrocław Probusa.." if GAID matches
            const filtered = filterOutages(mockOutages, 'Rozbrat', { streetGAID: 105 });
            expect(filtered.some(o => o.Message === 'Wrocław Probusa..')).toBe(true);
        });

        it('returns empty array when no match found', () => {
            const filtered = filterOutages(mockOutages, 'Main Street', { streetGAID: 999 });
            expect(filtered).toHaveLength(0);
        });

        it('returns empty array when street name is empty and no GAID match', () => {
            const filtered = filterOutages(mockOutages, '', { streetGAID: 999 });
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
