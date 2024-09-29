import re, sys

h = [ list( map( int, re.findall( "-?\d+", l ) ) )
      for l in sys.stdin ]

# The rock must be at the position of hailstone 0 at time 0:
#
#     px0 + vx0 t0 = pxr + vxr t0
#     py0 + vy0 t0 = pyr + vyr t0
#
# Solve for t0:
#
#     t0 = (pxr - px0) / (vx0 - vxr)
#     t0 = (pyr - py0) / (vy0 - vyr)
#
# Substitute first t0 definition into the second to eliminate it:
#
#     (pxr - px0) / (vx0 - vxr) = (pyr - py0) / (vy0 - vyr)
#
# Cross multiply, subtract right side from left, and expand:
#
#     pxr vy0 - pxr vyr + px0 vyr - px0 vy0 + py0 vx0 - py0 vxr + pyr vxr - pyr vx0 = 0
#
# Do the same for hailstone 1:
#
#     pxr vy1 - pxr vyr + px1 vyr - px1 vy1 + py1 vx1 - py1 vxr + pyr vxr - pyr vx1 = 0
#
# Subtract the equation of hailstone 1 from hailstone 0.  This eliminates the multiplied unknowns.
# Collect the unknowns pxr, pyr, vxr, vyr:
#
#     pxr (vy0 - vy1) + pyr (vx1 - vx0) + vxr (py1 - py0) + vyr (px0 - px1) - px0 vy0 + py0 vx0 + px1 vy1 - py1 vx1 = 0
#
# Since we have four unknowns, take the first four pairs of hailstones similarly:
#
#     pxr (vy0 - vy1) + pyr (vx1 - vx0) + vxr (py1 - py0) + vyr (px0 - px1) - px0 vy0 + py0 vx0 + px1 vy1 - py1 vx1 = 0
#     pxr (vy2 - vy3) + pyr (vx3 - vx2) + vxr (py3 - py2) + vyr (px2 - px3) - px2 vy2 + py2 vx2 + px3 vy3 - py3 vx3 = 0
#     pxr (vy4 - vy5) + pyr (vx5 - vx4) + vxr (py5 - py4) + vyr (px4 - px5) - px4 vy4 + py4 vx4 + px5 vy5 - py5 vx5 = 0
#     pxr (vy6 - vy7) + pyr (vx7 - vx6) + vxr (py7 - py6) + vyr (px6 - px7) - px6 vy6 + py6 vx6 + px7 vy7 - py7 vx7 = 0
#
# That gives us a nice linear system with four equations.  Cramer's rule (using integer math) takes care of it.
#
# Finding pzr is then mostly plug and chug with a 2x2 system from the first two hailstones.

px0, py0, pz0, vx0, vy0, vz0 = h[ 0 ]
px1, py1, pz1, vx1, vy1, vz1 = h[ 1 ]
px2, py2,   _, vx2, vy2,   _ = h[ 2 ]
px3, py3,   _, vx3, vy3,   _ = h[ 3 ]
px4, py4,   _, vx4, vy4,   _ = h[ 4 ]
px5, py5,   _, vx5, vy5,   _ = h[ 5 ]
px6, py6,   _, vx6, vy6,   _ = h[ 6 ]
px7, py7,   _, vx7, vy7,   _ = h[ 7 ]

def det3x3( m ):
    return ( m[ 0 ] * m[ 4 ] * m[ 8 ] + m[ 1 ] * m[ 5 ] * m[ 6 ] + m[ 2 ] * m[ 3 ] * m[ 7 ] -
             m[ 0 ] * m[ 5 ] * m[ 7 ] - m[ 1 ] * m[ 3 ] * m[ 8 ] - m[ 2 ] * m[ 4 ] * m[ 6 ] )
def det4x4( m ):
    return ( m[ 0 ] * det3x3( [ m[  5 ], m[  6 ], m[  7 ],
                                m[  9 ], m[ 10 ], m[ 11 ],
                                m[ 13 ], m[ 14 ], m[ 15 ] ] ) -
             m[ 1 ] * det3x3( [ m[  4 ], m[  6 ], m[  7 ],
                                m[  8 ], m[ 10 ], m[ 11 ],
                                m[ 12 ], m[ 14 ], m[ 15 ] ] ) +
             m[ 2 ] * det3x3( [ m[  4 ], m[  5 ], m[  7 ],
                                m[  8 ], m[  9 ], m[ 11 ],
                                m[ 12 ], m[ 13 ], m[ 15 ] ] ) -
             m[ 3 ] * det3x3( [ m[  4 ], m[  5 ], m[  6 ],
                                m[  8 ], m[  9 ], m[ 10 ],
                                m[ 12 ], m[ 13 ], m[ 14 ] ] ) )

A = [ vy0 - vy1, vx1 - vx0, py1 - py0, px0 - px1,
      vy2 - vy3, vx3 - vx2, py3 - py2, px2 - px3,
      vy4 - vy5, vx5 - vx4, py5 - py4, px4 - px5,
      vy6 - vy7, vx7 - vx6, py7 - py6, px6 - px7 ]
b = [ px0 * vy0 - py0 * vx0 + py1 * vx1 - px1 * vy1,
      px2 * vy2 - py2 * vx2 + py3 * vx3 - px3 * vy3,
      px4 * vy4 - py4 * vx4 + py5 * vx5 - px5 * vy5,
      px6 * vy6 - py6 * vx6 + py7 * vx7 - px7 * vy7 ]

den = det4x4( A )
pxr = det4x4( [ b[ 0 ], A[  1 ], A[  2 ], A[  3 ],
                b[ 1 ], A[  5 ], A[  6 ], A[  7 ],
                b[ 2 ], A[  9 ], A[ 10 ], A[ 11 ],
                b[ 3 ], A[ 13 ], A[ 14 ], A[ 15 ] ] ) // den
pyr = det4x4( [ A[  0 ], b[ 0 ], A[  2 ], A[  3 ],
                A[  4 ], b[ 1 ], A[  6 ], A[  7 ],
                A[  8 ], b[ 2 ], A[ 10 ], A[ 11 ],
                A[ 12 ], b[ 3 ], A[ 14 ], A[ 15 ] ] ) // den
vxr = det4x4( [ A[  0 ], A[  1 ], b[ 0 ], A[  3 ],
                A[  4 ], A[  5 ], b[ 1 ], A[  7 ],
                A[  8 ], A[  9 ], b[ 2 ], A[ 11 ],
                A[ 12 ], A[ 13 ], b[ 3 ], A[ 15 ] ] ) // den
vyr = det4x4( [ A[  0 ], A[  1 ], A[  2 ], b[ 0 ],
                A[  4 ], A[  5 ], A[  6 ], b[ 1 ],
                A[  8 ], A[  9 ], A[ 10 ], b[ 2 ],
                A[ 12 ], A[ 13 ], A[ 14 ], b[ 3 ] ] ) // den

t0 = ( pxr - px0 ) // ( vx0 - vxr )
t1 = ( pxr - px1 ) // ( vx1 - vxr )
vzr = ( pz0 - pz1 + t0 * vz0 - t1 * vz1 ) // ( t0 - t1 )
pzr = pz0 + t0 * ( vz0 - vzr )

print(f"{pxr=}, {pyr=}, {pzr=}")
print(f"{vxr=}, {vyr=}, {vzr=}")
print( pxr + pyr + pzr )
