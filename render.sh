for a in *.dot;
	do dot $a -Tpng > clips/$(echo $a | sed -e 's/\.dot//').png;
done
