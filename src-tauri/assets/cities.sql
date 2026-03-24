SELECT woj.nazwa woj, pow.nazwa pow, gmi.nazwa gmi, miasto.nazwa miasto, miasto.sym, osiedle.sym
FROM simc miasto
LEFT JOIN terc woj
on miasto.woj=woj.woj
and woj.pow is null
and woj.gmi is null
LEFT JOIN terc pow
on miasto.woj=pow.woj
and miasto.pow = pow.pow
and pow.gmi is null
LEFT JOIN terc gmi
on miasto.woj=gmi.woj
and miasto.pow = gmi.pow
and miasto.gmi = gmi.gmi
and miasto.rodz_gmi = gmi.rodz 
left join simc osiedle
on miasto.woj = osiedle.woj 
and miasto.pow = osiedle.pow
and osiedle.rodz_gmi in (8, 9)
and osiedle.rm not in (99,0)
where miasto.sym = miasto.sympod
and miasto.nazwa = 'Łódź';