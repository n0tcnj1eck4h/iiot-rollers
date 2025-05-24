$fn = 60;

*linear_extrude(3.5)
difference(){
    circle(5);
    circle(3.3 + 0.2);
}

translate([0,0, 30 + 8])
linear_extrude(10)
difference(){
    circle(32 / 2);
    circle(30.5 / 2);
}


translate([0,0,2])
linear_extrude(6)
difference(){
    circle(7 / 2);
    circle(3.4 / 2 -0.1);
}

intersection() {
    difference() {
        translate([0,0,8])
        cylinder(30, 3.5, 16);
        
        translate([0,0,7])
        cylinder(32, 1.3, 15.8);
    }
    
    union(){
        cube([40,10, 100], center=true);
        cube([10,40, 100], center=true);
    }
}