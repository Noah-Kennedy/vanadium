#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Created on Fri Jun 19 16:18:09 2020

@author: berisha
"""

from optir_utils import *
import argparse

'''Run python3 optir_proc.py --help from command line for usage help.

The following example, if run from command line, would create an annotation mask for the entire row (all stroma annotations)
using annotations from each core.

python3 optir_proc.py --rowmask 
                      --maskdir /Users/sbstn/Desktop/box/optir/csv/C/annotations/stroma 
                      --data /Users/sbstn/Desktop/box/optir/csv/C/C 
                      --cores 10 
                      --outdir /Users/sbstn/Desktop/box 
                      --filename row-c-stroma.png 
                      --tissuemask /Users/sbstn/Desktop/box/row-c-raw-tissue-mask-thresh-005.png
Args:
    --rowmask      indicating that an annotation image for the entire row should be created
    --maskidr      path to folder where the annotations for each core are
    --data         path to folder where the raw csv folders are, e.g. C1, C2, ..., C10 - this is necessary for padding
    --cores        number of total cores in the row
    --outdir       path to folder where the image will be saved
    --filename     name of image file
    --tissuemask   path to where the tissue mask file is -- this is multiplied elementwise with the annotations to make sure
                   annotated pixels correpond to real tissue pixels

1. 
python3 optir_proc.py --rowmosaic --image --mask --saveenvi --data /Users/sbstn/Box/research/optir/data/csv/C/C --cores 10 --thresh 0.05 --filename row-c --outdir /Users/sbstn/Box/research/optir/data/row-c-row-3/ --bands 1162 1234 1396 1540 1664 --vmax 1
2. 
python3 optir_proc.py --rowmask --split --maskdir /Users/sbstn/Box/research/optir/data/csv/C/annotations/epithelium/each-core --data /Users/sbstn/Box/research/optir/data/csv/C/C --cores 10 --outdir /Users/sbstn/Box/research/optir/data/row-c-row-3/ --filename row-c-epith.png --tissuemask /Users/sbstn/Box/research/optir/data/test/row-c-tissue-mask.png --colsplit 13500
3. 
python3 optir_proc.py --rowmask --split --maskdir /Users/sbstn/Box/research/optir/data/csv/C/annotations/stroma --data /Users/sbstn/Box/research/optir/data/csv/C/C --cores 10 --outdir /Users/sbstn/Box/research/optir/data/row-c-row-3/ --filename row-c-stroma.png --tissuemask /Users/sbstn/Box/research/optir/data/test/row-c-tissue-mask.png --colsplit 13500
4. 
python3 optir_proc.py --overlaps --maskdir /Users/sbstn/Box/research/optir/data/row-c-row-3/all
5. 
python3 optir_proc.py --overlaps --maskdir /Users/sbstn/Box/research/optir/data/row-c-row-3/train
6. 
python3 optir_proc.py --overlaps --maskdir /Users/sbstn/Box/research/optir/data/row-c-row-3/test
'''
parser = argparse.ArgumentParser(description="Tools for processing OPTIR data")

# required args
# required = parser.add_argument_group('required named arguments')
# required.add_argument("--data", help="Path to data folder.", type=str)
# required.add_argument("--cores", help="Num of cores per row", type=int)
# required.add_argument("--thresh", help="Threshold for keeping tissue areas", type=float)

# optional arguments
parser.add_argument("--data", help="Path to data folder.", type=str)
parser.add_argument("--cores", help="Num of cores per row", type=int)
parser.add_argument("--thresh", help="Threshold for keeping tissue areas", type=float)
parser.add_argument("--rowmosaic", help="Create row mosaic", action="store_true")
parser.add_argument("--outdir", help="Output directory", type=str)
parser.add_argument("--envi", help="Save it as envi", action="store_true")
parser.add_argument("--image", help="Save it as an image. Associated args: data, band, filename, outdir", action="store_true")
parser.add_argument("--filename", help="Name of file used for saving", type=str)
parser.add_argument("--mask", help="Generate and save a mask given a threshold", action='store_true')
parser.add_argument("--vmax", help="Maximum pixel value", type=int)
parser.add_argument("--vmin", help="Minimum pixel value", type=int)
parser.add_argument("--rowmask", help="Create a row mask from core masks", action='store_true')
parser.add_argument("--extension", help="Extension of mask files for each core", type=str, default=None)
parser.add_argument("--maskdir", help="Path to mask folder.", type=str)
parser.add_argument("--tissuemask", help="Path to tissue mask file.", type=str)
parser.add_argument("--split", help="Split mask into training and testing parts.", action='store_true')
parser.add_argument("--colsplit", help="Column index to be used for splitting masks.", type=int)
parser.add_argument("--saveenvi", help="Save ndarray to envi file.", action='store_true')
parser.add_argument("--bands", help="List of wavenumbers.", metavar='N', type=int, nargs='+')
parser.add_argument("--overlaps", help="Remove overlapping areas.", action='store_true')
parser.add_argument("--bkg", help="Background values.", metavar='N', type=float, nargs='+')
#parser.add_argument("--bkg1", help="Background values.", metavar='N', type=float, nargs='+')
parser.add_argument("--firstfile", help="Path to first file.", type=str)
parser.add_argument("--secondfile", help="Path to second file file.", type=str)
parser.add_argument("--envimosaic", help="Mosaic two envi files vertically.", action='store_true')
parser.add_argument("--maskmosaic", help="Mosaic two mask files vertically.", action='store_true')
parser.add_argument("--band", help="Band number.", type=int)
parser.add_argument("--normalize", help="Normalize to one band.", action="store_true")

#parser.add_argument("--bands", help="List of wavenumbers.", type=int, nargs='+')

args = parser.parse_args()

# /Users/sbstn/Desktop/box/optir/csv/C/C'

if args.rowmosaic:

    row_array = csv2row(args.data, args.cores, args.thresh, args.bkg)

    if args.image:
        save_image(args.filename, row_array, args.outdir, vmax=args.vmax)

    if args.mask:
        tissue_mask(row_array[:, :, 4], args.thresh, args.filename + '-tissue-mask-' + str(args.thresh) + '.png', args.outdir)

    if args.saveenvi:
        save_envi(args.filename + '.hdr', row_array, args.outdir, wavenumbers=args.bands)
elif args.mask:
    row_array = csv2row(args.data, args.cores, args.thresh)
    tissue_mask(row_array[:, :, 4], args.thresh, args.filename, args.outdir)
elif args.rowmask:
    if args.extension:
        make_row_mask(args.maskdir, args.data, args.cores, args.extension, args.outdir, args.filename, args.tissuemask,
                      args.split, args.colsplit)
    else:
        make_row_mask(args.maskdir, args.data, args.cores, args.outdir, args.filename, tissue_mask=args.tissuemask,
                      split=args.split, col_split=args.colsplit)
elif args.saveenvi:
    row_array = csv2row(args.data, args.cores, args.thresh)
    save_envi(args.filename, row_array, args.outdir, wavenumbers=args.bands)
elif args.overlaps:
    remove_overlaps(args.maskdir)
elif args.envimosaic:
    envi_mosaic(args.firstfile, args.secondfile, args.filename, args.outdir)
elif args.maskmosaic:
    mask_mosaic(args.firstfile, args.secondfile, args.filename, args.outdir)
elif args.image:
    save_band(args.data, args.band, args.filename, args.outdir)
elif args.normalize:
    normalize(args.data, args.band, args.filename, outdir=args.outdir)

'''
python3 optir_proc.py --envimosaic --firstfile /Users/msoesdl/Box/research/optir/data/rows-c-d-bkg/envi/rows-c-d-bkg.hdr --secondfile /Users/msoesdl/Box/research/optir/data/row-e-row-5-bkg/envi/row-e-bkg.hdr --outdir /Users/msoesdl/Box/research/optir/data/rows-c-e/ --filename rows-c-e-bkg.hdr

python3 optir_proc.py --maskmosaic --firstfile /Users/msoesdl/Box/research/optir/data/rows-c-d-bkg/test/rows-c-d-bkg-test-epith.png --secondfile /Users/msoesdl/Box/research/optir/data/row-e-row-5-bkg/test/corrected-test-row-e-bkg-epith.png --outdir /Users/msoesdl/Box/research/optir/data/rows-c-e/ --filename rows-c-e-bkg-test-epith.png

python3 optir_proc.py --image --data /Users/msoesdl/Box/research/optir/data/rows-c-e-bkg/envi/rows-c-e-bkg.hdr --band 4 --filename image-1664.png --outdir /Users/msoesdl/Box/research/optir/data/rows-c-e-bkg/

'''