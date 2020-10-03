#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Created on Fri Jun 19 10:23:23 2020

@author: berisha
"""
import os
import glob
import pandas as pd
import numpy as np
from tqdm import trange
import matplotlib.pyplot as plt
from skimage import io
from skimage import img_as_uint
from skimage.color import rgb2gray
from spectral import *
import imageio
from matplotlib import cm
import numpy as np
from matplotlib.colors import ListedColormap
# import python.envi


# matplotlib.use('Qt4Agg')
# matplotlib.use('TkAgg')

def csv2row(path, num_cores, thresh=None, bkg=None):
    """
    * Load csv raw cores of a row and stich them together.

    args:
        @path: path to directory including the prefix for each core folder
               example: /Users/msoesdl/Box/research/optir/csv/C/C
                        where core folders are organized as C1, C2, ... , C10
        @num_cores - number of cores in the row
        @thresh - optional threshold to be applied to remove non-tissue data
                - pixel values <= thresh are set to 0
                - currently it uses the last band (1664)

    returns:
        @row_max - maximum row dimension across all cores in the row
        @col_max - maximum col dimension across all cores in the row
        @row_array - numpy array of dimensions rows x cols x bands

    """
    extension = 'csv'

    # since cores from raw data have different dimension, we need to do some padding
    # so first let's get the maximum dimensions from all cores in the row
    # notice: this assumes that the dimensions in the same core are the same across bands for that core

    row_max, col_max = get_max_dims(path, num_cores)

    extension = 'csv'
    data_list = []

    # load all the csv cores into a list
    for i in trange(num_cores, desc='loading csv cores'):
        os.chdir(path + str(i + 1) + "/")
        # print('\n processing ', path + str(i + 1))
        all_filenames = [i for i in glob.glob('*.{}'.format(extension))]
        data_list.append([pd.read_csv(f, header=None).values for f in all_filenames])

    # pad the cores with zeros
    for i in trange(len(data_list), desc='padding'):
        dims = data_list[i][0].shape
        for j in range(len(data_list[i])):
            data_list[i][j] = np.pad(data_list[i][j], ((0, row_max - dims[0]), (0, col_max - dims[1])), 'constant',
                                     constant_values=0)

    # convert to np array and concat
    row_array = np.concatenate(data_list, axis=2)

    # transpose dimension so the resulting array is of size rows x cols x bands
    row_array = row_array.transpose(1, 2, 0)

    # plt.imsave('row-c-1664.png', row_array[:, :, 4], vmax=1)
    if bkg:
        bands = row_array.shape[2]
        for i in range(bands):
            print('\n bkg ', bkg[i])
            row_array[:, :, i] = row_array[:, :, i] / bkg[i]

    # if bkg:
    #     bands = row_array.shape[2]
    #     for i in range(bands):
    #         print('\n bkg ', bkg[i])
    #         row_array[:,0:8000,i] = row_array[:,0:8000,i]/bkg[i]
    #         print('\n bkg1 ', bkg1[i])
    #         row_array[:, 8000:, i] = row_array[:, 8000:, i] / bkg1[i]

    if thresh:
        row_array[row_array[:, :, 4] <= thresh, :] = 0

    return np.nan_to_num(row_array)


def get_max_dims(path, num_cores):
    '''
        Returns the max row and col dimensions from all cores in a row.

        args:
            @path - path to data dir
            @num_cores - total number of cores in a row

        returns:
            @row_max - max row dim
            @col_max - max col dim
    '''
    row_dims = []
    col_dims = []

    extension = 'csv'

    for i in trange(num_cores, desc='getting max row and col dims'):
        os.chdir(path + str(i + 1) + "/")
        all_filenames = [i for i in glob.glob('*.{}'.format(extension))]
        # load just the first csv
        temp = pd.read_csv(all_filenames[0], header=None).values.shape
        row_dims.append(temp[0])
        col_dims.append(temp[1])

    row_max = sorted(row_dims)[-1]
    col_max = sorted(col_dims)[-1]

    return row_max, col_max


def save_image(filename, array, outdir='.', vmin=None, vmax=None, extension='.png'):
    '''

    '''
    # save each band for debugging - this should be changed to saving only one band
    for i in range(array.shape[2]):
        temp = array[:, :, i]
        # change range to 0 - 255
        # temp = ((temp - temp.min()) * (1 / (temp.max() - temp.min()) * 255)).astype('uint8')

        plt.imsave(outdir + '/' + filename + '-' + str(i + 1) + extension, np.sqrt(temp), vmin=0, vmax=1,
                   cmap='coolwarm')

    plt.imsave(outdir + '/' + filename + '-' + 'all' + extension, np.sqrt(np.sum(array, axis=2)), cmap='coolwarm')


def tissue_mask(band, thresh, filename, outdir='.'):
    '''

    '''
    mask = np.zeros(band.shape, dtype=np.bool)
    mask[band > thresh] = 1
    # io.imsave(outdir + '/' + filename, img_as_uint(mask))
    imageio.imwrite(outdir + '/' + filename, (mask * 255).astype(np.uint8))


def save_envi(filename, array, outdir=None, wavenumbers=[]):
    ''' Save numpy array to envi file.

    Args:
        filename (str):     name of file in the format name.hdr
        array (ndarray):    numpy array of size r x c x b
        outdir (str):       path to output directory of the form /d1/d2/.../d, where d1, d2, d are dir names.
                                Notice: no forward slash at the end of path.
        wavelengths (list): list of wavenumbers corresponding to each slice of the array

    Returns:
        None.
    '''
    metadata = {}
    metadata['lines'] = array.shape[0]
    metadata['samples'] = array.shape[1]
    if wavenumbers:
        metadata['spectra names'] = wavenumbers

    if outdir:
        envi.save_image(outdir + '/' + filename, array.astype(np.float32), metadata=metadata, force=True, ext=None)
    else:
        envi.save_image(filename, array.astype(np.float32), metadata=metadata)


def open_envi(path, ):
    ''' Open an envi file and return an ndarray from it.

    Args:
        path (str): path to the .hdr file

    Returns:
        array (ndarray): of size rows x cols x bands
    '''

    img = open_image(path)
    return img[:, :, :]


def make_row_mask(mask_path, data_path, num_cores, outdir, filename, extension='png', tissue_mask=None, split=None,
                  col_split=None):
    '''


    Returns
    -------
    None.

    '''

    # load the masks
    row_max, col_max = get_max_dims(data_path, num_cores)

    mask_list = []

    os.chdir(mask_path)
    all_filenames = [i for i in glob.glob('*.{}'.format(extension))]
    all_filenames = sorted(all_filenames, key=lambda s: int(''.join(list(filter(str.isdigit, s)))))
    # mask_list.append([io.imread(f, as_gray=True) for f in all_filenames])
    mask_list.append([imageio.imread(f, as_gray=True).astype(np.bool) for f in all_filenames])

    # pad the masks with zeros
    for i in trange(len(mask_list), desc='padding'):
        for j in range(len(mask_list[i])):
            dims = mask_list[i][j].shape
            mask_list[i][j] = np.pad(mask_list[i][j], ((0, row_max - dims[0]), (0, col_max - dims[1])), 'constant',
                                     constant_values=0)

    # convert to np array and concat
    row_mask = np.concatenate(mask_list[0], axis=1)

    # make it binary
    # row_mask[row_mask > 0] = 1

    if tissue_mask:
        # tissue = io.imread(tissue_mask, as_gray=True)
        tissue = imageio.imread(tissue_mask, as_gray=True).astype(np.bool)
        tissue[tissue > 0] = 1
        row_mask = row_mask * tissue;

    # io.imsave(outdir + '/' + filename, img_as_uint(row_mask))
    imageio.imwrite(outdir + '/' + filename, (row_mask * 255).astype(np.uint8))

    # save mask
    if split:
        # split into training and testing - left for training, right for testing
        temp = row_mask.copy()
        temp[:, col_split:] = 0
        # io.imsave(outdir + '/' + 'train-' + filename, img_as_uint(temp))
        imageio.imwrite(outdir + '/' + 'train-' + filename, (temp * 255).astype(np.uint8))

        row_mask[:, 0:col_split] = 0;
        # io.imsave(outdir + '/' + 'test-' + filename, img_as_uint(temp))
        imageio.imwrite(outdir + '/' + 'test-' + filename, (row_mask * 255).astype(np.uint8))


def remove_overlaps(path, extension='png'):
    ''' Remove the overlapping areas in annotations.

    Args:
        path (str): path to folder where all annotations are stored for each class.

    Returns:
        Nothing. Saves annotations after removing overlaps.
    '''
    os.chdir(path)
    all_filenames = [i for i in glob.glob('*.{}'.format(extension))]

    dict = {}

    # load each image
    for f in all_filenames:
        temp = imageio.imread(f, as_gray=True)
        temp[temp > 0] = 1
        dict[f] = temp

    sum = np.zeros(dict[all_filenames[0]].shape)
    for key in dict:
        sum = sum + dict[key]

    # idx of overlapping pixels
    idx = sum > 1

    print('Number of overlapping pixels: ', np.count_nonzero(idx))

    # set overlapping areas to 0
    for key, value in dict.items():
        value[idx] = 0
        dict[key] = value

    for key in dict:
        imageio.imwrite('corrected-' + key, dict[key])


def get_colormap():
    '''

    Returns: Brewerish colormap
    '''
    # Oranges
    # PuRd
    # BuPu
    # PuBu
    # Purples
    # RdPu
    bottom = cm.get_cmap('Blues', 256)
    top = cm.get_cmap('Oranges', 256)

    # newcolors = np.vstack((bottom(np.linspace(0, 1, 256)),
    #                        top(np.linspace(0, 1, 256)),
    #                        np.asarray([0, 0, 0, 1])))
    # newcolors = np.vstack((np.asarray([0, 0, 0, 1]),
    #                        bottom(np.linspace(0, 1, 256)),
    #                        top(np.linspace(0, 1, 256))))

    newcolors = np.vstack((np.asarray([0, 0, 0, 1]),
                           bottom(np.linspace(0, 1, 256)),
                           top(np.linspace(0, 1, 256))
                           ))

    N = 256
    pink = np.ones((N, 4))
    # purple = np.ones((N, 4))
    blue = np.ones((N, 4))

    pink[:, 0] = np.linspace(199 / 256, 255 / 256, N)
    pink[:, 1] = np.linspace(20 / 256, 192 / 256, N)
    pink[:, 2] = np.linspace(133 / 256, 203 / 256, N)

    # purple[:, 0] = np.linspace(75/256, 216/256, N)
    # purple[:, 1] = np.linspace(0/256, 191/256, N)
    # purple[:, 2] = np.linspace(130/256, 216/256, N)

    blue[:, 0] = np.linspace(0 / 256, 200 / 256, N)
    blue[:, 1] = np.linspace(0 / 256, 200 / 256, N)
    blue[:, 2] = np.linspace(112 / 256, 255 / 256, N)

    pink = pink[::-1]
    blue = blue[::-1]

    # return ListedColormap(np.vstack((np.asarray([0, 0, 0, 1]), pink, blue)))

    return ListedColormap(newcolors, name='brewer')


def save_image_cmap(path, array, cmap):
    ''' Save a given 2d array to a color image using brewer cmap.

    Args:
        path (str): path to saving directory, including the filename
        array (ndarray): 2d array
        cmap : colormap

    Returns: Save image to given path.

    Notice: works only for 2 class for now.
    '''

    s = array.shape
    s = np.append(s, 3)
    rgb = np.zeros(s, dtype=np.ubyte)
    unique_values = np.unique(array)
    # TODO: generalize to multiple clases
    rgb[array == 1, :] = np.asarray([255, 0, 0])
    rgb[array == 2, :] = np.asarray([0, 255, 0])
    plt.imsave(path, rgb, cmap=cmap)


def save_prediction_maps(masks_path, data_path, clf, outdir, mc):
    ''' Generate and save classification maps.

    Args:
        masks_path (str):   path to mask directory
        data_path (str):    path to data dir including the envi file name without the .hdr
        clf (str):          classifier
        outdir (str):       path to output dir
        mc :                colormap

    Returns: Nothing. Saves masks to given outdir.

    '''

    os.chdir(masks_path)
    classimages = sorted(glob.glob('*.{}'.format('png')))  # load the class file names
    C = python.classify.filenames2class(classimages)  # generate the class images for testing
    C = C.astype(np.uint32)

    # get number of classes
    num_classes = C.shape[0]

    for i in range(1, num_classes):
        C[i, :, :] *= i + 1

    for i in range(num_classes):
        total_mask = C[i, :, :]
        test_set = python.envi.envi(data_path, mask=total_mask)
        N = np.count_nonzero(total_mask)  # set the batch size
        Tv = []  # initialize the target array to empty
        x_test = test_set.loadbatch(N)
        idx = np.flatnonzero(total_mask)  # get the indices of valid pixels
        y_test = total_mask.flat[idx]
        test_predictions = clf.predict(x_test.transpose())
        class_map = np.zeros(total_mask.shape)
        class_map[np.unravel_index(idx, total_mask.shape)] = test_predictions
        save_image_cmap(outdir + classimages[i].split('.')[0] + '-ground-truth.png', total_mask, mc)
        save_image_cmap(outdir + classimages[i].split('.')[0] + '-prediction.png', class_map, mc)
        # save_image_cmap(outdir + classimages[i].split('.')[0] + '-difference.png', class_map - total_mask, mc)
        t = class_map - total_mask
        plt.imsave(outdir + classimages[i].split('.')[0] + '-difference.png', t, cmap=mc)


def envi_mosaic(first_path, second_path, filename, outdir=None):
    ''' Mosaic two envi files together vertically.

    Args:
        first_path (str):   Path to first envi file, including the file name with the .hdr
        second_path (str):  Path to second envi file, including the file name with the .hdr
        outdir (str):       Path to output directory
        filename (str):     Name of file to save.

    Returns: None. Saves the new 'filename' envi to outdir.
    '''
    first_img = open_image(first_path).asarray().astype(np.float32)
    print('\n loaded first')
    print('\n first_img.max() ', first_img.max())
    second_img = open_image(second_path).asarray().astype(np.float32)
    print('\n loaded second')
    print('\n second_img.max() ', second_img.max())
    # first_memmap = first_img.open_memmap().astype(np.float32)
    # second_memmap = second_img.open_memmap().astype(np.float32)
    max_cols = max(first_img.shape[1], second_img.shape[1])
    # all = np.memmap(outdir + '/' + filename, dtype='float32', mode='w+', shape=(19349, 28740, 5))

    # all= np.concatenate((np.pad(first_img, ((0,0), (0, max_cols-first_img.shape[1]), (0,0)), 'constant', constant_values=0).astype(np.float32),
    #                      np.pad(second_img, ((0,0), (0, max_cols-second_img.shape[1]), (0,0)), 'constant', constant_values=0).astype(np.float32)),
    #                      axis=0)

    # all = np.zeros((11019, 28740, 5)).astype(np.float32)
    # print('\n all.dtype ', all.dtype)
    # print('\n all.max() ', all.max())
    # print('\n all.min() ', all.min())
    # print('\n all.shape ', all.shape)
    # print('\n np.count_nonzero(np.isinf(a)) ', np.count_nonzero(np.isinf(all)))

    print('\n new shape ', (first_img.shape[0] + second_img.shape[0], max_cols, first_img.shape[2]))
    img = envi.create_image(outdir + '/' + filename,
                            shape=(first_img.shape[0] + second_img.shape[0], max_cols, first_img.shape[2]),
                            dtype=np.float32,
                            force=True,
                            ext=None)

    mm = img.open_memmap(writable=True)
    mm[:, :, :] = np.concatenate((np.pad(first_img, ((0, 0), (0, max_cols - first_img.shape[1]), (0, 0)), 'constant',
                                         constant_values=0).astype(np.float32),
                                  np.pad(second_img, ((0, 0), (0, max_cols - second_img.shape[1]), (0, 0)), 'constant',
                                         constant_values=0).astype(np.float32)),
                                 axis=0)

    print('\n mm.shape ', mm.shape)
    print('\n mm.dtype ', mm.dtype)
    print('\n mm.max() ', mm.max())
    print('\n mm.min() ', mm.min())
    print('\n mm.shape ', mm.shape)
    print('\n np.count_nonzero(np.isinf(mm)) ', np.count_nonzero(np.isinf(mm)))

    del mm


def mask_mosaic(first_path, second_path, filename, outdir=None):
    ''' Mosaic two mask files together vertically.

    Args:
        first_path (str):   Path to first mask file, including the file name with the extension
        second_path (str):  Path to second envi file, including the file name with the extension
        outdir (str):       Path to output directory
        filename (str):     Name of file to save.

    Returns: None. Saves the new 'filename' envi to outdir.
    '''
    first_img = imageio.imread(first_path, as_gray=True).astype(np.bool)
    second_img = imageio.imread(second_path, as_gray=True).astype(np.bool)
    max_cols = max(first_img.shape[1], second_img.shape[1])

    all = np.concatenate(
        (np.pad(first_img, ((0, 0), (0, max_cols - first_img.shape[1])), 'constant', constant_values=0),
         np.pad(second_img, ((0, 0), (0, max_cols - second_img.shape[1])), 'constant', constant_values=0)),
        axis=0)

    if outdir:
        imageio.imsave(outdir + '/' + filename, (all * 255).astype(np.uint8))
    else:
        imageio.imsave(filename, (all * 255).astype(np.uint8))


def save_band(envi_file, band, filename, outdir=None):
    ''' Save band as image.
    Args:
        envi_file (str): path to envi file, including filename.hdr.
        band (int): band number
        filename (str): output file name.
        outdir (str): path to output dir

    Returns: None. Saves band as image.
    '''

    img = open_image(envi_file)
    if outdir:
        plt.imsave(outdir + '/' + filename, np.sqrt(np.squeeze(img[:, :, band])), vmax=1, cmap='coolwarm')
    else:
        plt.imsave(filename, np.sqrt(np.squeeze(img[:, :, band])), vmax=1, cmap='coolwarm')


def normalize(envi_file, band, filename, outdir=None):
    ''' Normalize data by band. Assumes envi is saved as bip with dimensions rxcxb.
    Args:
        envi_file (str): path to envi file, including filename.hdr.
        band (int): band number
        filename (str): output file name.
        outdir (str): path to output dir

    Returns: None. Saves band as image.
    '''

    img = open_image(envi_file).asarray().astype(np.float32)
    if outdir:
        new_envi = envi.create_image(outdir + '/' + filename,
                                shape=img.shape,
                                dtype=np.float32,
                                force=True,
                                ext=None)

        mm = new_envi.open_memmap(writable=True)
        temp = img[:,:,band]

        #mm[img>0] = img[img>0] / temp[:, :, np.newaxis]
        #mm[:, :, :] = np.divide(img[:, :, :], temp[:, :, np.newaxis], where=temp!=0)

        np.divide(img[:,:,:], temp[:, :, np.newaxis], out=mm[:,:,:], where=temp[:, :, np.newaxis] > 0)

        #mm[:,:,:] = np.divide(img[:, :, :], temp[:, :, np.newaxis], where=temp[:, :, np.newaxis] > 0)
        print('done')

    else:
        new_envi = envi.create_image(filename,
                                shape=img.shape,
                                dtype=np.float32,
                                force=True,
                                ext=None)

        mm = new_envi.open_memmap(writable=True)
        temp = img[:, :, band]
        #mm[:, :, :] = img[:, :, :] / temp[:, :, np.newaxis]
        #mm[img > 0] = img[img > 0] / temp[:, :, np.newaxis]
        #mm[:, :, :] = np.divide(img[:, :, :], temp[:, :, np.newaxis], where=temp[:,:,:] > 0)
        #np.divide(mm, temp[:, :, np.newaxis], out=mm[:,:,:], where=temp[:, :, np.newaxis] > 0)

        #out = np.where(t[:, :, np.newaxis] != 0, data / t[:, :, np.newaxis], 0)
        np.divide(img[:,:,:], temp[:, :, np.newaxis], out=mm[:,:,:], where=temp[:, :, np.newaxis] > 0)

    plt.imsave(outdir + '/' + filename + '-' + 'img-1664.png', mm[:, :, band], vmax=1, cmap='coolwarm')

if __name__ == '__main__':
    # row_max, col_max, row_array = csv2row('/Users/msoesdl/Box/research/optir/csv/C/C', 10)
    # row_max, col_max, row_array = csv2row('/Users/sbstn/Desktop/box/optir/csv/C/C', 10, 0.01)

    # tissue_mask(row_array[:,:,4], 0.01, 'test.png', '/Users/sbstn/Desktop/box')

    # make_row_mask('/Users/msoesdl/Box/research/optir/data/orig-masks/row-c-cores/epith/',
    #               '/Users/msoesdl/Box/research/optir/data/csv/C/C',
    #               10,
    #               '/Users/msoesdl/Box/research/optir/data/row-c-row-3-orig',
    #               'row-c-epith.png',
    #               tissue_mask='/Users/sbstn/Desktop/box/row-c-raw-tissue-mask-thresh-005.png',
    #               split=True,
    #               col_split=13500)

    # make_row_mask('/Users/sbstn/Box/research/optir/data/new-masks/masks/row-d-cores/epith/',
    #               '/Users/sbstn/Box/research/optir/data/csv/D/D',
    #               10,
    #               '/Users/sbstn/Box/research/optir/data/row-d-row-4/',
    #               'row-d-epith.png',
    #               tissue_mask='/Users/sbstn/Box/research/optir/data/row-d-row-4/row-d-tissue-mask-0.06.png',
    #               split=True,
    #               col_split=13500)

    # row_array = csv2row('/Users/sbstn/Desktop/box/optir/csv/C/C', 10, 0.05)
    # save_image('row-c-mosaic', row_array, '/Users/sbstn/Desktop/box', 1)
    # remove_overlaps('/Users/sbstn/Box/research/optir/data/test/all', extension='png')

    # envi_mosaic('/Users/msoesdl/Box/research/optir/data/row-c-row-3-bkg/envi/row-c-bkg.hdr',
    #             '/Users/msoesdl/Box/research/optir/data/row-d-row-4-bkg/envi/row-d-bkg.hdr',
    #             'test')

    # csv2row('/Users/msoesdl/Box/research/optir/data/csv/D/D', 10, thresh=0.05, bkg=None, bkg1=None)

    normalize('/Users/msoesdl/Box/research/optir/data/rows-c-j-bkg/envi/rows-c-j-bkg.hdr', \
              4, \
              'rows-c-j-bkg-norm.hdr', \
              outdir='/Users/msoesdl/Box/research/optir/data/rows-c-j-bkg/envi/')
