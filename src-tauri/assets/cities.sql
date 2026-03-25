SELECT woj.nazwa woj,
    pow.nazwa pow,
    gmi.nazwa gmi,
    miasto.nazwa miasto,
    miasto.sym
FROM simc miasto
    LEFT JOIN terc woj ON miasto.woj = woj.woj
    AND woj.pow is null
    AND woj.gmi is null
    LEFT JOIN terc pow ON miasto.woj = pow.woj
    AND miasto.pow = pow.pow
    AND pow.gmi is null
    LEFT JOIN terc gmi ON miasto.woj = gmi.woj
    AND miasto.pow = gmi.pow
    AND miasto.gmi = gmi.gmi
    AND miasto.rodz_gmi = gmi.rodz
WHERE miasto.sym = miasto.sympod
    AND miasto.nazwa = 'Poznań';