for a in *.dot;
	do sfdp $a -Tpng > clips/$(echo $a | sed -e 's/\.dot//').png;
done
