$(document).ready(function() {
    // Function to populate the combobox
    /*
    function populateCombobox() {
        $.ajax({
            url: '/students', // Endpoint to fetch options
            method: 'GET',
            dataType: 'json',
            success: function(data) {
                var combobox = $('#LN-0001');
                combobox.empty(); // Clear existing options
                $.each(data, function(index, item) {
                    combobox.append($('<option>', {
                        value: item.id,
                        text: item.name
                    }));
                });
            },
            error: function(xhr, status, error) {
                console.error("Error loading combobox options:", error);
            }
        });
    }
    */

    // Function to update content based on selection
    $('input[name^="N-"]').on('input', function() {
        //const list_name = "L" + $(this).attr('name');
        //const myCombobox = $('input[name="' + list_name + '"]');
        const myCombobox = document.getElementById($(this).attr('name'));
        var selectedValue = $(this).val();
        if (selectedValue && $.isNumeric(selectedValue)) {
            const myInput = $(this)[0];
            const myDatalist = document.getElementById($(this).attr('name'));

            let selectedLabel = null;
            const inputValue = myInput.value;

            // Find the option with the matching value (ID)
            for (const option of myDatalist.options) {
                if (option.value === inputValue) {
                    selectedLabel = option.textContent;
                    break;
                }
            }

            // If a matching option is found, update the input field for display
            if (selectedLabel) {
                myInput.value = selectedLabel;
                // Store the actual ID (inputValue) for later use, e.g., in a hidden input
                // document.getElementById('hiddenIdField').value = inputValue;
            }
        } else if (selectedValue && selectedValue.length >= 2) {
            $.ajax({
                url: '/students', // Endpoint to fetch content
                method: 'GET',
                data: { filter: selectedValue },
                dataType: "json",
                success: function(response) {
                    console.log(response);

                    // Iterate through the array
                    myCombobox.innerHTML = '';
                    $.each(response, function(_index, option) {
                        const newOption = document.createElement('option');
                        newOption.value = option.id;
                        newOption.textContent = option.name;
                        myCombobox.appendChild(newOption);
                    });
                },
                error: function(xhr, status, error) {
                    console.error("Error loading content:", error);
                }
            });
        } else {
            myCombobox.innerHTML = '';
        }
    });

    $('datalist').on('change', function() {
        const selectElement = document.getElementById($(this).attr('name'));
        const selectedOption = selectElement.options[selectElement.selectedIndex];

        const valueInput = selectedOption.value; // Get the value attribute of the selected option
        const idInput = selectedOption.id;     // Get the id attribute of the selected option

        console.log('valueInput: ', valueInput);
        console.log('idInput: ', idInput);

        $('input[name="' + $(this).attr('name') + '"]').textContent = valueInput;
    });

    // Initial population of the combobox on page load
    //populateCombobox();
});

function hide_empty() {
    // Iterate through each row in the table
    $('table').each(function() {
        var inputsInRow = $(this).find('input[type="text"][name^="N-"]');

        // Check if there are at least three input fields to determine "pre-previous"
        if (inputsInRow.length >= 2) {
            for (let i = inputsInRow.length - 1; i >= 1; i--) {
                var previousInputValue = $(inputsInRow[i - 1]).val();
                var currentInputValue = $(inputsInRow[i]).val();
                if (previousInputValue.trim() === '' && currentInputValue.trim() === '') {
                    $(inputsInRow[i]).parents('tr').hide();
                } else {
                    $(inputsInRow[i]).parents('tr').show();
                    break;
                }
            }
        }
    });
}

$(document).ready(function() {
    hide_empty();

    $(this).find('input[type="text"][name^="N-"]').on('input', function() {
        hide_empty();
    });
});
