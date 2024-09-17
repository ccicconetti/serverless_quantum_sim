# Excerpt of the utilities provided with HamLib available here:
# https://portal.nersc.gov/cfs/m888/dcamps/hamlib/hamlib_snippets.py


import numpy as np
import openfermion as of
import h5py


def parse_through_hdf5(func):
    """
    Decorator function that iterates through an HDF5 file and performs
    the action specified by ‘ func ‘ on the internal and leaf nodes in the
    HDF5 file.
    """

    def wrapper(obj, path="/", key=None):
        if type(obj) in [h5py._hl.group.Group, h5py._hl.files.File]:
            for ky in obj.keys():
                func(obj, path, key=ky, leaf=False)
                wrapper(obj=obj[ky], path=path + ky + ",", key=ky)
        elif type(obj) is h5py._hl.dataset.Dataset:
            func(obj, path, key=None, leaf=True)

    return wrapper


def get_hdf5_keys(fname_hdf5: str):
    """Get a list of keys to all datasets stored in the HDF5 file.

    Args
    ----
    fname_hdf5 ( str ) : full path where HDF5 file is stored
    """

    all_keys = []

    @parse_through_hdf5
    def action(obj, path="/", key=None, leaf=False):
        if leaf is True:
            all_keys.append(path)

    with h5py.File(fname_hdf5, "r") as f:
        action(f["/"])
    return all_keys


def read_openfermion_hdf5(fname_hdf5: str, key: str, optype=of.QubitOperator):
    """
    Read any openfermion operator object from HDF5 file at specified key.
    'optype' is the op class, can be of.QubitOperator or of.FermionOperator.
    """

    with h5py.File(fname_hdf5, "r", libver="latest") as f:
        op = optype(f[key][()].decode("utf-8"))
    return op
