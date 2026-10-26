//! The editing of a formula at a caret, a `CellPath`: the caret's moves, the edits a key makes, the
//! atom a command typed after `\` names, and where a caret stands on a formula's layout. The editor
//! holds the caret; each edit changes the formula in place and gives the caret's new place.

use crate::{Atom, AtomBox, CellPath, Formula, FormulaLayout, List, OPERATOR_NAMES, commands};

/// The list of the cell a path names, if it names one.
pub fn cell<'a>(f: &'a Formula, path: &CellPath) -> Option<&'a List> {
    let mut list = f.rows.get(path.row)?.get(path.column)?;
    for &(atom, c) in &path.steps {
        list = list.get(atom)?.cells().into_iter().nth(c)?;
    }
    Some(list)
}

/// The list of the cell a path names, mutable.
pub fn cell_mut<'a>(f: &'a mut Formula, path: &CellPath) -> Option<&'a mut List> {
    let mut list = f.rows.get_mut(path.row)?.get_mut(path.column)?;
    for &(atom, c) in &path.steps {
        list = list.get_mut(atom)?.cells_mut().into_iter().nth(c)?;
    }
    Some(list)
}

/// The atom right after a path's position.
pub fn atom_at<'a>(f: &'a Formula, path: &CellPath) -> Option<&'a Atom> {
    cell(f, path)?.get(path.offset)
}

/// The position before the atom whose cell holds the path, and the index of that cell; `None` in a
/// cell of the formula's grid.
fn parent(path: &CellPath) -> Option<(CellPath, usize)> {
    let mut up = path.clone();
    let (atom, c) = up.steps.pop()?;
    up.offset = atom;
    Some((up, c))
}

/// The position `offset` of the cell `c` of the atom after `at`.
fn child(at: &CellPath, c: usize, offset: usize) -> CellPath {
    let mut path = at.clone();
    path.steps.push((at.offset, c));
    path.offset = offset;
    path
}

fn with_offset(path: &CellPath, offset: usize) -> CellPath {
    CellPath {
        offset,
        ..path.clone()
    }
}

fn top(row: usize, column: usize, offset: usize) -> CellPath {
    CellPath {
        row,
        column,
        steps: Vec::new(),
        offset,
    }
}

/// The first position of a formula.
pub fn start() -> CellPath {
    CellPath::default()
}

/// The last position of a formula.
pub fn end(f: &Formula) -> CellPath {
    let row = f.rows.len().saturating_sub(1);
    let column = f.rows[row].len().saturating_sub(1);
    top(row, column, f.rows[row][column].len())
}

/// The path, brought into the formula: each step it names kept while it names a cell, and its
/// offset clamped to its cell, so a caret survives an edit that changed the formula's shape.
pub fn clamp(f: &Formula, path: &CellPath) -> CellPath {
    let row = path.row.min(f.rows.len() - 1);
    let column = path.column.min(f.rows[row].len() - 1);
    let mut out = top(row, column, 0);
    let mut list = &f.rows[row][column];
    for &(atom, c) in &path.steps {
        match list.get(atom).and_then(|a| a.cells().into_iter().nth(c)) {
            Some(inner) => {
                out.steps.push((atom, c));
                list = inner;
            }
            None => {
                out.offset = atom.min(list.len());
                return out;
            }
        }
    }
    out.offset = path.offset.min(list.len());
    out
}

/// The next cell of the formula's grid, row by row.
fn next_grid(f: &Formula, row: usize, column: usize) -> Option<(usize, usize)> {
    if column + 1 < f.rows[row].len() {
        Some((row, column + 1))
    } else if row + 1 < f.rows.len() {
        Some((row + 1, 0))
    } else {
        None
    }
}

/// The previous cell of the formula's grid.
fn previous_grid(f: &Formula, row: usize, column: usize) -> Option<(usize, usize)> {
    if column > 0 {
        Some((row, column - 1))
    } else if row > 0 {
        Some((row - 1, f.rows[row - 1].len() - 1))
    } else {
        None
    }
}

/// The cell Right enters an atom by, as LyX's `idxFirst`: a root's index where it has one, else
/// the first cell.
fn first_cell(atom: &Atom) -> usize {
    match atom {
        Atom::Radical { index: Some(_), .. } => 1,
        _ => 0,
    }
}

/// The cell Left enters an atom by, as LyX's `idxLast`: a root's body, else the last cell.
fn last_cell(atom: &Atom) -> usize {
    match atom {
        Atom::Radical { .. } => 0,
        _ => atom.cells().len() - 1,
    }
}

/// The cell Right goes to from the end of the cell `c` of an atom, as LyX's `idxForward`: a
/// matrix's next column in the same row, a root's body from its index; `None` for every other
/// atom, which Right leaves, so a fraction's denominator and a script are reached by Up, Down or
/// Tab.
fn cell_forward(atom: &Atom, c: usize) -> Option<usize> {
    match atom {
        Atom::Matrix { rows, .. } => {
            let (r, k) = grid_place(rows, c)?;
            (k + 1 < rows[r].len()).then(|| grid_index(rows, r, k + 1))
        }
        Atom::Radical { index: Some(_), .. } => (c == 1).then_some(0),
        _ => None,
    }
}

/// The cell Left goes to from the start of the cell `c` of an atom, as LyX's `idxBackward`, the
/// mirror of [`cell_forward`].
fn cell_backward(atom: &Atom, c: usize) -> Option<usize> {
    match atom {
        Atom::Matrix { rows, .. } => {
            let (r, k) = grid_place(rows, c)?;
            (k > 0).then(|| grid_index(rows, r, k - 1))
        }
        Atom::Radical { index: Some(_), .. } => (c == 0).then_some(1),
        _ => None,
    }
}

/// The row and the column of the cell `c` of a matrix, as `Atom::cells` numbers its cells, row by
/// row, rows of unequal length counted as they are.
fn grid_place(rows: &[Vec<List>], c: usize) -> Option<(usize, usize)> {
    let mut c = c;
    for (r, row) in rows.iter().enumerate() {
        if c < row.len() {
            return Some((r, c));
        }
        c -= row.len();
    }
    None
}

/// The index, as `Atom::cells` numbers them, of the cell at row `r` and column `k` of a matrix.
fn grid_index(rows: &[Vec<List>], r: usize, k: usize) -> usize {
    rows[..r].iter().map(Vec::len).sum::<usize>() + k
}

/// The position Right gives, as LyX's `Cursor::mathForward`, `design@rootpoint-math@lyx-caret-moves`:
/// past a symbol; into an atom with cells
/// at the start of the cell [`first_cell`] names; at a cell's end, to the cell
/// [`cell_forward`] names, or out of the atom; at the end of a cell of the formula's grid, to the
/// next column of its row; `None` at the end of a row's last cell, where the caret leaves the
/// formula, as LyX's leaves a formula's grid.
pub fn right(f: &Formula, p: &CellPath) -> Option<CellPath> {
    let list = cell(f, p)?;
    if let Some(atom) = list.get(p.offset) {
        return Some(if atom.cells().is_empty() {
            with_offset(p, p.offset + 1)
        } else {
            child(p, first_cell(atom), 0)
        });
    }
    match parent(p) {
        Some((up, c)) => Some(match cell_forward(atom_at(f, &up)?, c) {
            Some(next) => child(&up, next, 0),
            None => with_offset(&up, up.offset + 1),
        }),
        None => (p.column + 1 < f.rows[p.row].len()).then(|| top(p.row, p.column + 1, 0)),
    }
}

/// The position Left gives, the mirror of [`right`]: into an atom with cells at the end of the
/// cell [`last_cell`] names, and at a cell's start to the cell [`cell_backward`] names, or before
/// the atom.
pub fn left(f: &Formula, p: &CellPath) -> Option<CellPath> {
    if p.offset > 0 {
        let at = with_offset(p, p.offset - 1);
        let atom = atom_at(f, &at)?;
        if atom.cells().is_empty() {
            return Some(at);
        }
        let c = last_cell(atom);
        return Some(child(&at, c, atom.cells()[c].len()));
    }
    match parent(p) {
        Some((up, c)) => {
            let atom = atom_at(f, &up)?;
            Some(match cell_backward(atom, c) {
                Some(previous) => child(&up, previous, atom.cells()[previous].len()),
                None => up,
            })
        }
        None => p
            .column
            .checked_sub(1)
            .map(|k| top(p.row, k, f.rows[p.row][k].len())),
    }
}

/// Every position of a formula, cell by cell in the order of `Atom::cells`, row by row of the grid:
/// the positions Right and Left reach, and those they pass by, as a fraction's denominator.
pub fn positions(f: &Formula) -> Vec<CellPath> {
    fn walk(list: &List, at: &CellPath, out: &mut Vec<CellPath>) {
        for offset in 0..=list.len() {
            out.push(with_offset(at, offset));
        }
        for (i, atom) in list.iter().enumerate() {
            for (c, inner) in atom.cells().into_iter().enumerate() {
                walk(inner, &child(&with_offset(at, i), c, 0), out);
            }
        }
    }
    let mut out = Vec::new();
    for (r, row) in f.rows.iter().enumerate() {
        for (k, list) in row.iter().enumerate() {
            walk(list, &top(r, k, 0), &mut out);
        }
    }
    out
}

/// The cell above (`up`) or below the cell `c` of an atom, if the atom has one there: a fraction's
/// numerator and denominator; a script from its nucleus, only from the nucleus's end (`at_end`),
/// as LyX's `InsetMathScript::idxUpDown`, and the nucleus from a script; the rows of a matrix, in
/// the nearest column of a shorter row.
fn vertical(atom: &Atom, c: usize, up: bool, at_end: bool) -> Option<usize> {
    match atom {
        Atom::Fraction { .. } => match (c, up) {
            (1, true) => Some(0),
            (0, false) => Some(1),
            _ => None,
        },
        Atom::Scripts { sub, sup, .. } => {
            let sub_at = sub.as_ref().map(|_| 1);
            let sup_at = sup.as_ref().map(|_| 1 + usize::from(sub.is_some()));
            match (Some(c), up) {
                (Some(0), true) if at_end => sup_at,
                (Some(0), false) if at_end => sub_at,
                (s, true) if s == sub_at => Some(0),
                (s, false) if s == sup_at => Some(0),
                _ => None,
            }
        }
        Atom::Matrix { rows, .. } => {
            let (r, k) = grid_place(rows, c)?;
            let r2 = if up { r.checked_sub(1)? } else { r + 1 };
            let row = rows.get(r2).filter(|row| !row.is_empty())?;
            Some(grid_index(rows, r2, k.min(row.len() - 1)))
        }
        _ => None,
    }
}

/// The position Up (`up`) or Down gives: the nearest atom holding the caret that has a cell above
/// or below the caret's, then the rows of the formula; `None` where there is none.
pub fn vertical_move(f: &Formula, p: &CellPath, up: bool) -> Option<CellPath> {
    for k in (0..p.steps.len()).rev() {
        let at = CellPath {
            steps: p.steps[..k].to_vec(),
            offset: p.steps[k].0,
            ..p.clone()
        };
        let atom = atom_at(f, &at)?;
        let (from, innermost) = (p.steps[k].1, k + 1 == p.steps.len());
        let at_end = innermost && p.offset == atom.cells()[from].len();
        if let Some(c) = vertical(atom, from, up, at_end) {
            let len = atom.cells()[c].len();
            // From a script back to its nucleus, at the nucleus's end.
            let offset = match atom {
                Atom::Scripts { .. } if c == 0 => len,
                _ if innermost => p.offset,
                _ => 0,
            };
            return Some(child(&at, c, offset.min(len)));
        }
    }
    let row = if up {
        p.row.checked_sub(1)?
    } else {
        Some(p.row + 1).filter(|r| *r < f.rows.len())?
    };
    let column = p.column.min(f.rows[row].len() - 1);
    let offset = if p.steps.is_empty() { p.offset } else { 0 };
    Some(top(row, column, offset.min(f.rows[row][column].len())))
}

/// The position Tab gives, as LyX's `cell-forward` of `InsetMathNest`, in the atom whose cell holds
/// the caret: the start of its next cell, of its first after its last, and, in an atom of one cell,
/// the position after it; in the formula's grid the same, row by row, and `None` in a grid of one
/// cell, where the caret leaves the formula.
pub fn next_cell(f: &Formula, p: &CellPath) -> Option<CellPath> {
    match parent(p) {
        Some((up, c)) => {
            let n = atom_at(f, &up)?.cells().len();
            Some(match (c + 1 < n, n) {
                (true, _) => child(&up, c + 1, 0),
                (false, 1) => with_offset(&up, up.offset + 1),
                (false, _) => child(&up, 0, 0),
            })
        }
        None => match next_grid(f, p.row, p.column) {
            Some((r, c)) => Some(top(r, c, 0)),
            None if f.rows.len() == 1 && f.rows[0].len() == 1 => None,
            None => Some(top(0, 0, 0)),
        },
    }
}

/// The position Space gives: after the atom whose cell holds the caret; `None` in a cell of the
/// grid, where the caret leaves the formula.
pub fn leave_cell(p: &CellPath) -> Option<CellPath> {
    parent(p).map(|(up, _)| with_offset(&up, up.offset + 1))
}

/// Insert an atom at the caret, and give the caret's new place: in an atom with cells, the start of
/// its first cell; otherwise after it.
pub fn insert(f: &mut Formula, p: &CellPath, atom: Atom) -> Option<CellPath> {
    let has_cells = !atom.cells().is_empty();
    cell_mut(f, p)?.insert(p.offset, atom);
    Some(if has_cells {
        child(p, 0, 0)
    } else {
        with_offset(p, p.offset + 1)
    })
}

/// Make the atom before the caret the nucleus of a superscript (`sup`) or a subscript, and put the
/// caret in the script: a script it has already is entered at its end, a group gives its list as
/// the nucleus, as LaTeX reads `{ab}^2`, and at a cell's start the nucleus is empty.
pub fn script(f: &mut Formula, p: &CellPath, sup: bool) -> Option<CellPath> {
    let list = cell_mut(f, p)?;
    let at = p.offset.checked_sub(1);
    let place = match at.map(|i| &mut list[i]) {
        Some(Atom::Scripts { sub: s, sup: t, .. }) => {
            let slot = if sup { t } else { s };
            if slot.is_none() {
                *slot = Some(Vec::new());
            }
            at.expect("a previous atom")
        }
        Some(_) => {
            let i = at.expect("a previous atom");
            let nucleus = match list.remove(i) {
                Atom::Group(inner) => inner,
                other => vec![other],
            };
            list.insert(i, scripts(nucleus, sup));
            i
        }
        None => {
            list.insert(0, scripts(Vec::new(), sup));
            0
        }
    };
    let Atom::Scripts { sub, sup: t, .. } = &list[place] else {
        unreachable!("the atom at `place` is a script");
    };
    let (c, len) = if sup {
        (
            1 + usize::from(sub.is_some()),
            t.as_ref().map_or(0, Vec::len),
        )
    } else {
        (1, sub.as_ref().map_or(0, Vec::len))
    };
    Some(child(&with_offset(p, place), c, len))
}

fn scripts(nucleus: List, sup: bool) -> Atom {
    Atom::Scripts {
        nucleus,
        sub: (!sup).then(Vec::new),
        sup: sup.then(Vec::new),
    }
}

/// The innermost matrix holding the caret: the position before it, and the index of the caret's
/// cell in it.
fn matrix_of(f: &Formula, p: &CellPath) -> Option<(CellPath, usize)> {
    let mut path = p.clone();
    while let Some((up, c)) = parent(&path) {
        if let Some(Atom::Matrix { .. }) = atom_at(f, &up) {
            return Some((up, c));
        }
        path = up;
    }
    None
}

/// `&`: in a matrix, the start of the next cell, if there is one; in a cell of the formula's grid,
/// an alignment point, the cell cut at the caret into two of the row. Elsewhere nothing.
pub fn ampersand(f: &mut Formula, p: &CellPath) -> Option<CellPath> {
    if let Some((at, c)) = matrix_of(f, p) {
        let n = atom_at(f, &at)?.cells().len();
        return (c + 1 < n).then(|| child(&at, c + 1, 0));
    }
    if !p.steps.is_empty() {
        return None;
    }
    let row = f.rows.get_mut(p.row)?;
    let tail = row.get_mut(p.column)?.split_off(p.offset);
    row.insert(p.column + 1, tail);
    Some(top(p.row, p.column + 1, 0))
}

/// Ctrl+Return: in a matrix, a new row of empty cells after the caret's, the caret at its start; in
/// a cell of the formula's grid, a new row, which takes what follows the caret. Elsewhere nothing.
pub fn new_row(f: &mut Formula, p: &CellPath) -> Option<CellPath> {
    if let Some((at, c)) = matrix_of(f, p) {
        let Some(Atom::Matrix { rows, .. }) = cell_mut(f, &at)?.get_mut(at.offset) else {
            return None;
        };
        let (r, _) = grid_place(rows, c)?;
        let columns = pad(rows);
        rows.insert(r + 1, vec![Vec::new(); columns]);
        return Some(child(&at, (r + 1) * columns, 0));
    }
    if !p.steps.is_empty() {
        return None;
    }
    let row = f.rows.get_mut(p.row)?;
    let mut moved = vec![row.get_mut(p.column)?.split_off(p.offset)];
    moved.extend(row.drain(p.column + 1..));
    f.rows.insert(p.row + 1, moved);
    Some(top(p.row + 1, 0, 0))
}

/// The edits of a matrix's rows and columns, LyX's Alt+M C and W: append an empty one
/// after the caret's, delete the caret's, copy it after itself, or swap it with the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixEdit {
    Append,
    Delete,
    Copy,
    Swap,
}

/// Edit the rows (`rows`) or the columns of the matrix holding the caret, and give the caret's new
/// place, in the same cell where it still exists; `None` with no matrix, or for a deletion of a
/// matrix's last row or column, or a swap of its last.
pub fn edit_matrix(
    f: &mut Formula,
    p: &CellPath,
    rows_not_columns: bool,
    edit: MatrixEdit,
) -> Option<CellPath> {
    let (at, c) = matrix_of(f, p)?;
    let Some(Atom::Matrix { rows, .. }) = cell_mut(f, &at)?.get_mut(at.offset) else {
        return None;
    };
    let (r, k) = grid_place(rows, c)?;
    let columns = pad(rows);
    let (mut r2, mut k2) = (r, k);
    if rows_not_columns {
        match edit {
            MatrixEdit::Append => rows.insert(r + 1, vec![Vec::new(); columns]),
            MatrixEdit::Delete if rows.len() > 1 => {
                rows.remove(r);
                r2 = r.min(rows.len() - 1);
            }
            MatrixEdit::Copy => rows.insert(r + 1, rows[r].clone()),
            MatrixEdit::Swap if r + 1 < rows.len() => rows.swap(r, r + 1),
            _ => return None,
        }
    } else {
        match edit {
            MatrixEdit::Append => rows
                .iter_mut()
                .for_each(|row| row.insert(k + 1, Vec::new())),
            MatrixEdit::Delete if columns > 1 => {
                rows.iter_mut().for_each(|row| {
                    row.remove(k);
                });
                k2 = k.min(columns - 2);
            }
            MatrixEdit::Copy => rows.iter_mut().for_each(|row| {
                let copy = row[k].clone();
                row.insert(k + 1, copy);
            }),
            MatrixEdit::Swap if k + 1 < columns => {
                rows.iter_mut().for_each(|row| row.swap(k, k + 1))
            }
            _ => return None,
        }
    }
    let columns = rows.first().map_or(1, Vec::len).max(1);
    let index = r2 * columns + k2;
    let offset = rows[r2][k2].len().min(p.offset);
    Some(child(&at, index, offset))
}

/// Give every row of a matrix as many cells as its longest, the new ones empty, as LaTeX reads a
/// row with fewer `&`, so an edit of its rows and columns can index each row alike; the number of
/// columns.
fn pad(rows: &mut [Vec<List>]) -> usize {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
    for row in rows.iter_mut() {
        row.resize(columns, Vec::new());
    }
    columns
}

/// What a press of Backspace or Delete does at a caret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Deletion {
    /// A symbol was deleted, or two cells of the grid joined; the caret's new place.
    Deleted(CellPath),
    /// The atom with cells after this position is selected, which a second press deletes.
    Select(CellPath),
    /// At the formula's first position for Backspace, or its last for Delete.
    Nothing,
}

/// Backspace (`forward` false) or Delete at a caret: a symbol goes; an atom with cells, or the atom
/// whose cell's edge the caret is at, is selected first; at a cell's edge in the formula's grid the
/// two cells, or the two rows, join; at the formula's first or last position nothing happens.
pub fn delete(f: &mut Formula, p: &CellPath, forward: bool) -> Option<Deletion> {
    let len = cell(f, p)?.len();
    let at = if forward {
        (p.offset < len).then_some(p.offset)
    } else {
        p.offset.checked_sub(1)
    };
    if let Some(i) = at {
        let target = with_offset(p, i);
        if !atom_at(f, &target)?.cells().is_empty() {
            return Some(Deletion::Select(target));
        }
        cell_mut(f, p)?.remove(i);
        return Some(Deletion::Deleted(target));
    }
    if let Some((up, _)) = parent(p) {
        return Some(Deletion::Select(up));
    }
    let (row, column) = (p.row, p.column);
    if forward {
        if column + 1 < f.rows[row].len() {
            let next = f.rows[row].remove(column + 1);
            f.rows[row][column].extend(next);
        } else if row + 1 < f.rows.len() {
            let mut next = f.rows.remove(row + 1).into_iter();
            if let Some(first) = next.next() {
                f.rows[row][column].extend(first);
            }
            f.rows[row].extend(next);
        } else {
            return Some(Deletion::Nothing);
        }
        Some(Deletion::Deleted(p.clone()))
    } else {
        let (r, c) = match previous_grid(f, row, column) {
            Some(rc) => rc,
            None => return Some(Deletion::Nothing),
        };
        let offset = f.rows[r][c].len();
        if r == row {
            let this = f.rows[row].remove(column);
            f.rows[r][c].extend(this);
        } else {
            let mut this = f.rows.remove(row).into_iter();
            if let Some(first) = this.next() {
                f.rows[r][c].extend(first);
            }
            f.rows[r].extend(this);
        }
        Some(Deletion::Deleted(top(r, c, offset)))
    }
}

/// Delete the atom after `at`, as a second press does on a selected atom.
pub fn delete_atom(f: &mut Formula, at: &CellPath) -> Option<CellPath> {
    let list = cell_mut(f, at)?;
    (at.offset < list.len()).then(|| {
        list.remove(at.offset);
        at.clone()
    })
}

/// The atom a command typed after `\` names: a command of the table or one of its absent
/// characters, an operator name, or a structure with its cells empty, as `\frac` and `\hat` are,
/// read as the parser reads `\frac{}{}`; an unknown name is an atom of its own, drawn red, as the
/// parser reads one. `None` for a name the subset reads as no single atom, as `left`.
pub fn command_atom(name: &str) -> Option<Atom> {
    for arguments in ["", "{}", "{}{}"] {
        let Ok(f) = Formula::parse(&format!("\\{name}{arguments}")) else {
            continue;
        };
        if let [row] = &f.rows[..]
            && let [cell] = &row[..]
            && let [atom] = &cell[..]
        {
            return Some(atom.clone());
        }
    }
    None
}

/// The atom a typed character inserts, as the parser reads it alone: a letter or a digit as a
/// symbol, an operator with its class. `None` for a character that is no atom alone, as `^`.
pub fn typed_atom(ch: char) -> Option<Atom> {
    let f = Formula::parse(&ch.to_string()).ok()?;
    match &f.rows[..] {
        [row] => match &row[..] {
            [cell] => match &cell[..] {
                [atom @ Atom::Symbol { .. }] => Some(atom.clone()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// The names of the structures a command typed after `\` makes, beside the table's symbols and the
/// operator names, which completion offers.
const STRUCTURES: &[&str] = &[
    "acute",
    "bar",
    "breve",
    "check",
    "ddot",
    "dot",
    "frac",
    "grave",
    "hat",
    "mathbb",
    "mathbf",
    "mathcal",
    "mathit",
    "mathrm",
    "overline",
    "qquad",
    "quad",
    "sqrt",
    "text",
    "tilde",
    "underline",
    "vec",
];

/// The names a command being typed may complete to: the table's commands, the operator names and
/// the structures, that start with `prefix`, sorted.
pub fn completions(prefix: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = commands()
        .iter()
        .map(|c| c.name)
        .chain(OPERATOR_NAMES.iter().map(|(name, _)| *name))
        .chain(STRUCTURES.iter().copied())
        .filter(|n| n.starts_with(prefix))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// The longest prefix the names starting with `prefix` share: what Tab completes a command to.
pub fn complete(prefix: &str) -> String {
    let names = completions(prefix);
    let Some(first) = names.first() else {
        return prefix.to_owned();
    };
    let mut end = first.len();
    for name in &names[1..] {
        end = end.min(
            first
                .chars()
                .zip(name.chars())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a.len_utf8())
                .sum(),
        );
    }
    first[..end].to_owned()
}

/// Where a caret stands on a formula's layout: the x of its line, and the cell it stands in, whose
/// baseline is at `y`, with its ascent and descent, in the layout's units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretBox {
    pub x: f64,
    pub y: f64,
    pub ascent: f64,
    pub descent: f64,
}

/// The caret box of a position on a formula's layout: at the left of the atom after it, or at the
/// right of the atom before it at a cell's end, or at the left of an empty cell's placeholder; the
/// height that of the cell. `None` for a position the layout lacks.
pub fn caret_box(layout: &FormulaLayout, p: &CellPath) -> Option<CaretBox> {
    let cell_path = with_offset(p, 0);
    let c = layout.cells.iter().find(|c| c.path == cell_path)?;
    let x = match layout.atoms.iter().find(|a| a.path == *p) {
        Some(a) => a.x,
        None => match p.offset.checked_sub(1) {
            Some(i) => {
                let before = with_offset(p, i);
                let a = layout.atoms.iter().find(|a| a.path == before)?;
                a.x + a.width
            }
            None => c.x,
        },
    };
    Some(CaretBox {
        x,
        y: c.y,
        ascent: c.ascent,
        descent: c.descent,
    })
}

/// The boxes of the atoms that hold a position, outer first: the frames the overlay draws.
pub fn holding_atoms<'a>(layout: &'a FormulaLayout, p: &CellPath) -> Vec<&'a AtomBox> {
    (0..p.steps.len())
        .filter_map(|k| {
            let at = CellPath {
                steps: p.steps[..k].to_vec(),
                offset: p.steps[k].0,
                ..p.clone()
            };
            layout.atoms.iter().find(|a| a.path == at)
        })
        .collect()
}

/// The position a point of a formula's layout gives: in the smallest cell whose box holds the point,
/// or the nearest cell of the grid where none does, before the first atom of that cell whose middle
/// is right of the point.
pub fn path_at(layout: &FormulaLayout, x: f64, y: f64) -> Option<CellPath> {
    // A caret at a cell's end stands at its right edge, which the sums of its atoms' widths reach
    // only to rounding.
    const EDGE: f64 = 1e-6;
    let holds = |c: &&crate::CellBox| {
        x >= c.x - EDGE
            && x <= c.x + c.width + EDGE
            && y >= c.y - c.ascent - EDGE
            && y <= c.y + c.descent + EDGE
    };
    let distance = |c: &crate::CellBox| {
        let dx = (c.x - x).max(x - c.x - c.width).max(0.0);
        let dy = (c.y - c.ascent - y).max(y - c.y - c.descent).max(0.0);
        dx * dx + dy * dy
    };
    let c = layout
        .cells
        .iter()
        .filter(holds)
        // The smallest cell holding the point is the innermost one there: a script's cell beside a
        // nucleus that a font change nests deeper is smaller than that nucleus.
        .min_by(|a, b| {
            let area = |c: &crate::CellBox| c.width * (c.ascent + c.descent);
            area(a).total_cmp(&area(b))
        })
        .or_else(|| {
            layout
                .cells
                .iter()
                .filter(|c| c.path.steps.is_empty())
                .min_by(|a, b| distance(a).total_cmp(&distance(b)))
        })?;
    let offset = layout
        .atoms
        .iter()
        .filter(|a| {
            a.path.row == c.path.row
                && a.path.column == c.path.column
                && a.path.steps == c.path.steps
        })
        .filter(|a| a.x + a.width / 2.0 < x)
        .count();
    Some(with_offset(&c.path, offset))
}
