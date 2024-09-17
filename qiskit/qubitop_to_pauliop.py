# Modified version of code from here:
# https://github.com/tsuvihatu/openfermion-qiskit/blob/main/openfermionqiskit/qiskit_covertor.py


from openfermion.ops import QubitOperator
from qiskit.quantum_info import SparsePauliOp


def qubitop_to_pauliop(qubit_operator):
    """Convert an openfermion QubitOperator to a Qiskit SparsePauliOp.
    Args:
        qubit_operator ("QubitOperator"): Openfermion QubitOperator to convert to a qiskit.quantum_info.SparsePauliOp.
    Returns:
        paulis ("SparsePauliOp"): Qiskit SparsePauliOp.
    """
    if not isinstance(qubit_operator, QubitOperator):
        raise TypeError("qubit_operator must be an openFermion QubitOperator object.")

    n_qubits = 0
    for qubit_terms, _ in qubit_operator.terms.items():
        for tensor_term in qubit_terms:
            assert len(tensor_term) == 2
            n_qubits = max(n_qubits, tensor_term[0] + 1)

    paulis = []
    for qubit_terms, coefficient in qubit_operator.terms.items():
        pauli_label_list = ["I" for _ in range(n_qubits)]

        for tensor_term in qubit_terms:
            assert tensor_term[0] < n_qubits
            pauli_label_list[tensor_term[0]] = tensor_term[1]

        pauli_label = ""
        for label in pauli_label_list:
            pauli_label += label
        paulis.append((pauli_label, coefficient))

    return SparsePauliOp.from_list(paulis)
