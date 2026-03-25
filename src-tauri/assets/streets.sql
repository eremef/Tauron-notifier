SELECT a.nazwa miasto,
    b.nazwa "część",
    u.cecha,
    u.nazwa_2,
    u.nazwa_1
FROM simc a
    LEFT JOIN simc b ON a.sym = b.sympod
    LEFT JOIN ulic u ON a.sym = u.sym
    OR b.sym = u.sym
WHERE a.sym = a.sympod
    AND u.sym_ul is not null
    AND a.sym = 969400