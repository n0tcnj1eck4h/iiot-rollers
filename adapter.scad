$fn=60;

translate([0,0,-7])
linear_extrude(7)
difference(){
    circle(4);
    circle(1.5);
}

linear_extrude(10)

difference(){
    circle(4);
    intersection(){
        circle(3);
        translate([1.5,0,0]) square(6, center=true);
    }
}