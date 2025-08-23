
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
