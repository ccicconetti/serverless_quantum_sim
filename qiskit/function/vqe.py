# Modified version of the example code provided by Qiskit serverless for VQE.
# The original code is available on GitHub:
# https://github.com/Qiskit/qiskit-serverless

import time
import numpy as np
from scipy.optimize import minimize

from qiskit_ibm_runtime import (
    EstimatorV2 as Estimator,
    SamplerV2 as Sampler,
    Session,
)
from qiskit_ibm_runtime import EstimatorV2 as Estimator, SamplerV2 as Sampler
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit_serverless import (
    get_arguments,
    save_result,
)

# Lists filled during the execution of the minimization process
COSTS = []
TIMES = []


def build_callback(ansatz, hamiltonian, estimator, callback_dict):
    """Return callback function that uses Estimator instance,
    and stores intermediate values into a dictionary.

    Parameters:
        ansatz (QuantumCircuit): Parameterized ansatz circuit
        hamiltonian (SparsePauliOp): Operator representation of Hamiltonian
        estimator (Estimator): Estimator primitive instance
        callback_dict (dict): Mutable dict for storing values

    Returns:
        Callable: Callback function object
    """

    def callback(current_vector):
        """Callback function storing previous solution vector,
        computing the intermediate cost value, and displaying number
        of completed iterations and average time per iteration.

        Values are stored in pre-defined 'callback_dict' dictionary.

        Parameters:
            current_vector (ndarray): Current vector of parameters
                                      returned by optimizer
        """

        # Keep track of the number of iterations
        callback_dict["iters"] += 1

        # Set the prev_vector to the latest one
        callback_dict["prev_vector"] = current_vector

        # Grab the current time
        current_time = time.perf_counter()

        # Find the total time spent outside of this callback
        # (only after the 1st iteration)
        if callback_dict["iters"] > 1:
            delta = current_time - callback_dict["_prev_time"]
            callback_dict["total_time"] += delta
            callback_dict["exec_times"].append(delta)

        # Set the previous time to the current time
        callback_dict["_prev_time"] = current_time

        # Print to screen on single line
        print(
            "({},{},{})".format(callback_dict["iters"], COSTS[-1], TIMES[-1]),
            end="\r",
            flush=True,
        )

    return callback


def cost_func(params, ansatz, hamiltonian, estimator):
    """Return estimate of energy from estimator

    Parameters:
        params (ndarray): Array of ansatz parameters
        ansatz (QuantumCircuit): Parameterized ansatz circuit
        hamiltonian (SparsePauliOp): Operator representation of Hamiltonian
        estimator (Estimator): Estimator primitive instance

    Returns:
        float: Energy estimate
    """

    current_time = time.perf_counter()
    energy = estimator.run([(ansatz, hamiltonian, params)]).result()[0].data.evs
    COSTS.append(float(energy))
    TIMES.append(time.perf_counter() - current_time)
    return energy


def run_vqe(initial_parameters, ansatz, operator, estimator, method, maxiter):
    callback_dict = {
        "prev_vector": None,
        "iters": 0,
        "total_time": 0,
        "exec_times": [],
        "_prev_time": None,
    }
    callback = build_callback(ansatz, operator, estimator, callback_dict)
    result = minimize(
        cost_func,
        initial_parameters,
        args=(ansatz, operator, estimator),
        method=method,
        callback=callback,
        options={"maxiter": maxiter},
    )
    return result, callback_dict


if __name__ == "__main__":
    durations = {}
    ts_last = time.perf_counter()
    arguments = get_arguments()

    service = arguments.get("service")
    ansatz = arguments.get("ansatz")
    operator = arguments.get("operator")
    method = arguments.get("method", "COBYLA")
    maxiter = int(arguments.get("maxiter", "1000"))
    initial_parameters = arguments.get("initial_parameters")
    if service:
        backend = service.least_busy(operational=True, simulator=False)
    else:
        from qiskit_aer import AerSimulator

        backend = AerSimulator()
    if initial_parameters is None:
        initial_parameters = 2 * np.pi * np.random.rand(ansatz.num_parameters)

    ts_cur = time.perf_counter()
    durations["run_init"] = ts_cur - ts_last
    ts_last = ts_cur

    pm = generate_preset_pass_manager(backend=backend, optimization_level=1)
    ansatz_isa = pm.run(ansatz)
    operator_isa = operator.apply_layout(ansatz_isa.layout)

    ts_cur = time.perf_counter()
    durations["run_transpile"] = ts_cur - ts_last
    ts_last = ts_cur

    if service:
        with Session(service=service, backend=backend) as session:
            estimator = Estimator(mode=session)
            vqe_result, callback_dict = run_vqe(
                initial_parameters=initial_parameters,
                ansatz=ansatz_isa,
                operator=operator_isa,
                estimator=estimator,
                method=method,
                maxiter=maxiter,
            )
    else:
        estimator = Estimator(backend=backend)
        vqe_result, callback_dict = run_vqe(
            initial_parameters=initial_parameters,
            ansatz=ansatz_isa,
            operator=operator_isa,
            estimator=estimator,
            method=method,
            maxiter=maxiter,
        )

    ts_cur = time.perf_counter()
    durations["run_vqe"] = ts_cur - ts_last
    ts_last = ts_cur

    qc = ansatz.assign_parameters(vqe_result.x)
    qc.measure_all()
    qc_isa = pm.run(qc)

    ts_cur = time.perf_counter()
    durations["run_qc"] = ts_cur - ts_last
    ts_last = ts_cur

    if service:
        with Session(service=service, backend=backend) as session:
            sampler = Sampler(mode=session)
            samp_dist = (
                sampler.run([qc_isa], shots=int(1e4)).result()[0].data.meas.get_counts()
            )
    else:
        sampler = Sampler(backend=backend)
        samp_dist = (
            sampler.run([qc_isa], shots=int(1e4)).result()[0].data.meas.get_counts()
        )

    ts_cur = time.perf_counter()
    durations["run_sampler"] = ts_cur - ts_last
    ts_last = ts_cur

    save_result(
        {
            "result": samp_dist,
            "optimal_point": vqe_result.x.tolist(),
            "optimal_value": vqe_result.fun,
            "optimizer_time": callback_dict.get("total_time", 0),
            "durations": durations,
            "times": TIMES,
            "costs": COSTS,
            "num_iterations": callback_dict.get("iters", 0),
        }
    )
